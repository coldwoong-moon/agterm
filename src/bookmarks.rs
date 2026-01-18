//! Bookmark system for AgTerm terminal emulator
//!
//! Provides a comprehensive bookmark system for saving and organizing
//! frequently used commands with support for:
//! - Named bookmarks with descriptions
//! - Working directory context
//! - Tag-based organization
//! - Usage tracking and analytics
//! - Search and filtering
//! - File persistence (JSON)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a single bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    /// Unique identifier
    pub id: Uuid,
    /// Display name of the bookmark
    pub name: String,
    /// The command to execute
    pub command: String,
    /// Optional working directory
    pub working_dir: Option<PathBuf>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// When the bookmark was created
    pub created_at: DateTime<Utc>,
    /// When the bookmark was last used
    pub last_used: Option<DateTime<Utc>>,
    /// Number of times the bookmark has been used
    pub use_count: u32,
    /// Optional description
    pub description: Option<String>,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(
        name: impl Into<String>,
        command: impl Into<String>,
        working_dir: Option<PathBuf>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            command: command.into(),
            working_dir,
            tags,
            created_at: Utc::now(),
            last_used: None,
            use_count: 0,
            description: None,
        }
    }

    /// Add a description to the bookmark
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Record usage of this bookmark
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(Utc::now());
    }

    /// Check if the bookmark matches a search query
    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.command.to_lowercase().contains(&query_lower)
            || self
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }
}

/// Updates that can be applied to a bookmark
#[derive(Debug, Clone, Default)]
pub struct BookmarkUpdate {
    pub name: Option<String>,
    pub command: Option<String>,
    pub working_dir: Option<Option<PathBuf>>,
    pub tags: Option<Vec<String>>,
    pub description: Option<Option<String>>,
}

impl BookmarkUpdate {
    /// Create a new empty update
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the command
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the working directory
    pub fn working_dir(mut self, working_dir: Option<PathBuf>) -> Self {
        self.working_dir = Some(working_dir);
        self
    }

    /// Set the tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set the description
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = Some(description);
        self
    }

    /// Apply the update to a bookmark
    pub fn apply_to(&self, bookmark: &mut Bookmark) {
        if let Some(ref name) = self.name {
            bookmark.name = name.clone();
        }
        if let Some(ref command) = self.command {
            bookmark.command = command.clone();
        }
        if let Some(ref working_dir) = self.working_dir {
            bookmark.working_dir = working_dir.clone();
        }
        if let Some(ref tags) = self.tags {
            bookmark.tags = tags.clone();
        }
        if let Some(ref description) = self.description {
            bookmark.description = description.clone();
        }
    }
}

/// Manages bookmarks with CRUD operations and analytics
pub struct BookmarkManager {
    bookmarks: HashMap<Uuid, Bookmark>,
    tag_index: HashMap<String, Vec<Uuid>>, // tag -> bookmark IDs
    file_path: Option<PathBuf>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new() -> Self {
        Self {
            bookmarks: HashMap::new(),
            tag_index: HashMap::new(),
            file_path: None,
        }
    }

    /// Create a bookmark manager with default bookmarks
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // Git bookmarks
        let _ = manager.add_bookmark(
            "Git Status",
            "git status",
            None,
            vec!["git".to_string(), "status".to_string()],
            Some("Check the status of the git repository"),
        );

        let _ = manager.add_bookmark(
            "Git Log",
            "git log --oneline --graph --decorate --all",
            None,
            vec!["git".to_string(), "log".to_string()],
            Some("View git commit history with graph"),
        );

        let _ = manager.add_bookmark(
            "Git Diff",
            "git diff",
            None,
            vec!["git".to_string(), "diff".to_string()],
            Some("Show changes in working directory"),
        );

        let _ = manager.add_bookmark(
            "Git Pull",
            "git pull --rebase",
            None,
            vec!["git".to_string(), "sync".to_string()],
            Some("Pull latest changes with rebase"),
        );

        // Docker bookmarks
        let _ = manager.add_bookmark(
            "Docker PS",
            "docker ps -a",
            None,
            vec!["docker".to_string(), "containers".to_string()],
            Some("List all containers"),
        );

        let _ = manager.add_bookmark(
            "Docker Images",
            "docker images",
            None,
            vec!["docker".to_string(), "images".to_string()],
            Some("List all Docker images"),
        );

        let _ = manager.add_bookmark(
            "Docker Clean",
            "docker system prune -a --volumes",
            None,
            vec!["docker".to_string(), "cleanup".to_string()],
            Some("Clean up Docker system (removes all unused data)"),
        );

        // System bookmarks
        let _ = manager.add_bookmark(
            "Disk Usage",
            "du -h -d 1 | sort -h",
            None,
            vec!["system".to_string(), "disk".to_string()],
            Some("Show disk usage sorted by size"),
        );

        let _ = manager.add_bookmark(
            "Process Tree",
            "ps aux --forest",
            None,
            vec!["system".to_string(), "processes".to_string()],
            Some("Display process tree"),
        );

        let _ = manager.add_bookmark(
            "Network Connections",
            "netstat -tuln",
            None,
            vec!["system".to_string(), "network".to_string()],
            Some("Show active network connections"),
        );

        // Rust development bookmarks
        let _ = manager.add_bookmark(
            "Cargo Build Release",
            "cargo build --release",
            None,
            vec!["rust".to_string(), "build".to_string()],
            Some("Build project in release mode"),
        );

        let _ = manager.add_bookmark(
            "Cargo Test",
            "cargo test",
            None,
            vec!["rust".to_string(), "test".to_string()],
            Some("Run all tests"),
        );

        let _ = manager.add_bookmark(
            "Cargo Check",
            "cargo check --all-features",
            None,
            vec!["rust".to_string(), "check".to_string()],
            Some("Check code with all features enabled"),
        );

        let _ = manager.add_bookmark(
            "Cargo Clippy",
            "cargo clippy --all-targets -- -D warnings",
            None,
            vec!["rust".to_string(), "lint".to_string()],
            Some("Run Clippy linter with warnings as errors"),
        );

        // File operations
        let _ = manager.add_bookmark(
            "Find Large Files",
            "find . -type f -size +100M -exec ls -lh {} \\;",
            None,
            vec!["files".to_string(), "search".to_string()],
            Some("Find files larger than 100MB"),
        );

        let _ = manager.add_bookmark(
            "Count Lines of Code",
            "find . -name '*.rs' | xargs wc -l | tail -1",
            None,
            vec!["files".to_string(), "stats".to_string()],
            Some("Count total lines of Rust code"),
        );

        manager
    }

    /// Add a new bookmark
    pub fn add_bookmark(
        &mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        working_dir: Option<PathBuf>,
        tags: Vec<String>,
        description: Option<impl Into<String>>,
    ) -> Result<Uuid, BookmarkError> {
        let mut bookmark = Bookmark::new(name, command, working_dir, tags.clone());

        if let Some(desc) = description {
            bookmark.description = Some(desc.into());
        }

        let id = bookmark.id;

        // Update tag index
        for tag in &tags {
            self.tag_index
                .entry(tag.to_lowercase())
                .or_default()
                .push(id);
        }

        self.bookmarks.insert(id, bookmark);

        tracing::debug!("Added bookmark with id: {}", id);
        Ok(id)
    }

    /// Remove a bookmark by ID
    pub fn remove_bookmark(&mut self, id: Uuid) -> Result<Bookmark, BookmarkError> {
        let bookmark = self
            .bookmarks
            .remove(&id)
            .ok_or(BookmarkError::NotFound(id))?;

        // Remove from tag index
        for tag in &bookmark.tags {
            if let Some(ids) = self.tag_index.get_mut(&tag.to_lowercase()) {
                ids.retain(|&i| i != id);
            }
        }

        tracing::debug!("Removed bookmark with id: {}", id);
        Ok(bookmark)
    }

    /// Update a bookmark
    pub fn update_bookmark(
        &mut self,
        id: Uuid,
        update: BookmarkUpdate,
    ) -> Result<(), BookmarkError> {
        let bookmark = self
            .bookmarks
            .get_mut(&id)
            .ok_or(BookmarkError::NotFound(id))?;

        // If tags are being updated, update the tag index
        if let Some(ref new_tags) = update.tags {
            // Remove old tags from index
            for tag in &bookmark.tags {
                if let Some(ids) = self.tag_index.get_mut(&tag.to_lowercase()) {
                    ids.retain(|&i| i != id);
                }
            }

            // Add new tags to index
            for tag in new_tags {
                self.tag_index
                    .entry(tag.to_lowercase())
                    .or_default()
                    .push(id);
            }
        }

        update.apply_to(bookmark);

        tracing::debug!("Updated bookmark with id: {}", id);
        Ok(())
    }

    /// Get a bookmark by ID
    pub fn get_bookmark(&self, id: Uuid) -> Option<&Bookmark> {
        self.bookmarks.get(&id)
    }

    /// List all bookmarks
    pub fn list_bookmarks(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<_> = self.bookmarks.values().collect();
        bookmarks.sort_by(|a, b| a.name.cmp(&b.name));
        bookmarks
    }

    /// Search bookmarks by query
    pub fn search_bookmarks(&self, query: &str) -> Vec<&Bookmark> {
        let mut results: Vec<_> = self
            .bookmarks
            .values()
            .filter(|b| b.matches_query(query))
            .collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// Get bookmarks by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&Bookmark> {
        let tag_lower = tag.to_lowercase();
        self.tag_index
            .get(&tag_lower)
            .map(|ids| {
                let mut bookmarks: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.bookmarks.get(id))
                    .collect();
                bookmarks.sort_by(|a, b| a.name.cmp(&b.name));
                bookmarks
            })
            .unwrap_or_default()
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<_> = self.tag_index.keys().cloned().collect();
        tags.sort();
        tags
    }

    /// Get most recently used bookmarks
    pub fn get_recent(&self, limit: usize) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<_> = self
            .bookmarks
            .values()
            .filter(|b| b.last_used.is_some())
            .collect();

        bookmarks.sort_by(|a, b| {
            b.last_used
                .unwrap_or(DateTime::<Utc>::MIN_UTC)
                .cmp(&a.last_used.unwrap_or(DateTime::<Utc>::MIN_UTC))
        });

        bookmarks.into_iter().take(limit).collect()
    }

    /// Get most frequently used bookmarks
    pub fn get_most_used(&self, limit: usize) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<_> = self
            .bookmarks
            .values()
            .filter(|b| b.use_count > 0)
            .collect();

        bookmarks.sort_by(|a, b| {
            b.use_count
                .cmp(&a.use_count)
                .then_with(|| b.last_used.cmp(&a.last_used))
        });

        bookmarks.into_iter().take(limit).collect()
    }

    /// Record usage of a bookmark
    pub fn record_use(&mut self, id: Uuid) -> Result<(), BookmarkError> {
        let bookmark = self
            .bookmarks
            .get_mut(&id)
            .ok_or(BookmarkError::NotFound(id))?;

        bookmark.record_use();

        tracing::debug!(
            "Recorded use of bookmark '{}' (total: {})",
            bookmark.name,
            bookmark.use_count
        );
        Ok(())
    }

    /// Get total number of bookmarks
    pub fn len(&self) -> usize {
        self.bookmarks.len()
    }

    /// Check if the manager has no bookmarks
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
        self.tag_index.clear();
        tracing::debug!("Cleared all bookmarks");
    }

    /// Save bookmarks to a JSON file
    pub fn save_to_file(&self, path: PathBuf) -> Result<(), BookmarkError> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BookmarkError::IoError(format!("Failed to create directory: {e}")))?;
        }

        let bookmarks: Vec<&Bookmark> = self.bookmarks.values().collect();
        let json = serde_json::to_string_pretty(&bookmarks)
            .map_err(|e| BookmarkError::SerializationError(e.to_string()))?;

        fs::write(&path, json)
            .map_err(|e| BookmarkError::IoError(format!("Failed to write file: {e}")))?;

        tracing::info!("Saved {} bookmarks to {:?}", bookmarks.len(), path);
        Ok(())
    }

    /// Load bookmarks from a JSON file
    pub fn load_from_file(&mut self, path: PathBuf) -> Result<(), BookmarkError> {
        if !path.exists() {
            // Create parent directory and empty file if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    BookmarkError::IoError(format!("Failed to create directory: {e}"))
                })?;
            }
            fs::write(&path, "[]")
                .map_err(|e| BookmarkError::IoError(format!("Failed to create file: {e}")))?;
            self.file_path = Some(path);
            return Ok(());
        }

        let json = fs::read_to_string(&path)
            .map_err(|e| BookmarkError::IoError(format!("Failed to read file: {e}")))?;

        // Handle empty files as empty bookmark list
        let json = json.trim();
        let bookmarks: Vec<Bookmark> = if json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(json)
                .map_err(|e| BookmarkError::SerializationError(e.to_string()))?
        };

        self.clear();
        for bookmark in bookmarks {
            let id = bookmark.id;
            let tags = bookmark.tags.clone();

            // Update tag index
            for tag in &tags {
                self.tag_index
                    .entry(tag.to_lowercase())
                    .or_default()
                    .push(id);
            }

            self.bookmarks.insert(id, bookmark);
        }

        self.file_path = Some(path.clone());
        tracing::info!("Loaded {} bookmarks from {:?}", self.bookmarks.len(), path);
        Ok(())
    }

    /// Get the file path if set
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Save to the configured file path
    pub fn save(&self) -> Result<(), BookmarkError> {
        if let Some(path) = &self.file_path {
            self.save_to_file(path.clone())
        } else {
            Err(BookmarkError::NoFilePath)
        }
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during bookmark operations
#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("Bookmark not found: {0}")]
    NotFound(Uuid),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("No file path configured")]
    NoFilePath,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark::new(
            "Test Bookmark",
            "ls -la",
            Some(PathBuf::from("/home")),
            vec!["test".to_string()],
        );

        assert_eq!(bookmark.name, "Test Bookmark");
        assert_eq!(bookmark.command, "ls -la");
        assert_eq!(bookmark.working_dir, Some(PathBuf::from("/home")));
        assert_eq!(bookmark.tags, vec!["test"]);
        assert_eq!(bookmark.use_count, 0);
        assert!(bookmark.last_used.is_none());
        assert!(bookmark.description.is_none());
    }

    #[test]
    fn test_bookmark_with_description() {
        let bookmark = Bookmark::new("Test", "echo hello", None, vec![])
            .with_description("A test bookmark");

        assert_eq!(bookmark.description, Some("A test bookmark".to_string()));
    }

    #[test]
    fn test_bookmark_record_use() {
        let mut bookmark = Bookmark::new("Test", "echo hello", None, vec![]);
        assert_eq!(bookmark.use_count, 0);
        assert!(bookmark.last_used.is_none());

        bookmark.record_use();
        assert_eq!(bookmark.use_count, 1);
        assert!(bookmark.last_used.is_some());

        bookmark.record_use();
        assert_eq!(bookmark.use_count, 2);
    }

    #[test]
    fn test_bookmark_matches_query() {
        let bookmark = Bookmark::new(
            "Git Status",
            "git status",
            None,
            vec!["git".to_string()],
        )
        .with_description("Check repository status");

        assert!(bookmark.matches_query("git"));
        assert!(bookmark.matches_query("status"));
        assert!(bookmark.matches_query("repository"));
        assert!(bookmark.matches_query("Git")); // case insensitive
        assert!(!bookmark.matches_query("docker"));
    }

    #[test]
    fn test_bookmark_manager_add() {
        let mut manager = BookmarkManager::new();

        let id = manager
            .add_bookmark(
                "Test",
                "echo test",
                None,
                vec!["test".to_string()],
                Some("description"),
            )
            .unwrap();

        assert_eq!(manager.len(), 1);
        assert!(manager.get_bookmark(id).is_some());
    }

    #[test]
    fn test_bookmark_manager_remove() {
        let mut manager = BookmarkManager::new();

        let id = manager
            .add_bookmark("Test", "echo test", None, vec![], None::<String>)
            .unwrap();

        assert_eq!(manager.len(), 1);

        let removed = manager.remove_bookmark(id).unwrap();
        assert_eq!(removed.name, "Test");
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_bookmark_manager_remove_not_found() {
        let mut manager = BookmarkManager::new();
        let result = manager.remove_bookmark(Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_bookmark_manager_update() {
        let mut manager = BookmarkManager::new();

        let id = manager
            .add_bookmark("Old Name", "old command", None, vec![], None::<String>)
            .unwrap();

        let update = BookmarkUpdate::new()
            .name("New Name")
            .command("new command")
            .description(Some("New description".to_string()));

        manager.update_bookmark(id, update).unwrap();

        let bookmark = manager.get_bookmark(id).unwrap();
        assert_eq!(bookmark.name, "New Name");
        assert_eq!(bookmark.command, "new command");
        assert_eq!(bookmark.description, Some("New description".to_string()));
    }

    #[test]
    fn test_bookmark_manager_list() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark("Zebra", "echo zebra", None, vec![], None::<String>)
            .unwrap();
        manager
            .add_bookmark("Apple", "echo apple", None, vec![], None::<String>)
            .unwrap();
        manager
            .add_bookmark("Mango", "echo mango", None, vec![], None::<String>)
            .unwrap();

        let bookmarks = manager.list_bookmarks();
        assert_eq!(bookmarks.len(), 3);
        // Should be sorted by name
        assert_eq!(bookmarks[0].name, "Apple");
        assert_eq!(bookmarks[1].name, "Mango");
        assert_eq!(bookmarks[2].name, "Zebra");
    }

    #[test]
    fn test_bookmark_manager_search() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark("Git Status", "git status", None, vec![], None::<String>)
            .unwrap();
        manager
            .add_bookmark("Git Commit", "git commit", None, vec![], None::<String>)
            .unwrap();
        manager
            .add_bookmark("Docker PS", "docker ps", None, vec![], None::<String>)
            .unwrap();

        let results = manager.search_bookmarks("git");
        assert_eq!(results.len(), 2);

        let results = manager.search_bookmarks("docker");
        assert_eq!(results.len(), 1);

        let results = manager.search_bookmarks("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_bookmark_manager_get_by_tag() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(
                "Git Status",
                "git status",
                None,
                vec!["git".to_string()],
                None::<String>,
            )
            .unwrap();
        manager
            .add_bookmark(
                "Git Log",
                "git log",
                None,
                vec!["git".to_string()],
                None::<String>,
            )
            .unwrap();
        manager
            .add_bookmark(
                "Docker PS",
                "docker ps",
                None,
                vec!["docker".to_string()],
                None::<String>,
            )
            .unwrap();

        let git_bookmarks = manager.get_by_tag("git");
        assert_eq!(git_bookmarks.len(), 2);

        let docker_bookmarks = manager.get_by_tag("docker");
        assert_eq!(docker_bookmarks.len(), 1);

        // Case insensitive
        let git_bookmarks_upper = manager.get_by_tag("GIT");
        assert_eq!(git_bookmarks_upper.len(), 2);
    }

    #[test]
    fn test_bookmark_manager_get_all_tags() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(
                "Test1",
                "cmd1",
                None,
                vec!["git".to_string(), "dev".to_string()],
                None::<String>,
            )
            .unwrap();
        manager
            .add_bookmark(
                "Test2",
                "cmd2",
                None,
                vec!["docker".to_string()],
                None::<String>,
            )
            .unwrap();

        let tags = manager.get_all_tags();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"git".to_string()));
        assert!(tags.contains(&"dev".to_string()));
        assert!(tags.contains(&"docker".to_string()));
    }

    #[test]
    fn test_bookmark_manager_get_recent() {
        let mut manager = BookmarkManager::new();

        let id1 = manager
            .add_bookmark("First", "cmd1", None, vec![], None::<String>)
            .unwrap();
        let id2 = manager
            .add_bookmark("Second", "cmd2", None, vec![], None::<String>)
            .unwrap();
        let id3 = manager
            .add_bookmark("Third", "cmd3", None, vec![], None::<String>)
            .unwrap();

        // Use them in specific order
        manager.record_use(id1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.record_use(id2).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.record_use(id3).unwrap();

        let recent = manager.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].name, "Third"); // Most recent
        assert_eq!(recent[1].name, "Second");
    }

    #[test]
    fn test_bookmark_manager_get_most_used() {
        let mut manager = BookmarkManager::new();

        let id1 = manager
            .add_bookmark("Low", "cmd1", None, vec![], None::<String>)
            .unwrap();
        let id2 = manager
            .add_bookmark("High", "cmd2", None, vec![], None::<String>)
            .unwrap();
        let id3 = manager
            .add_bookmark("Medium", "cmd3", None, vec![], None::<String>)
            .unwrap();

        // Use them different amounts
        manager.record_use(id1).unwrap();
        manager.record_use(id2).unwrap();
        manager.record_use(id2).unwrap();
        manager.record_use(id2).unwrap();
        manager.record_use(id3).unwrap();
        manager.record_use(id3).unwrap();

        let most_used = manager.get_most_used(3);
        assert_eq!(most_used.len(), 3);
        assert_eq!(most_used[0].name, "High"); // 3 uses
        assert_eq!(most_used[1].name, "Medium"); // 2 uses
        assert_eq!(most_used[2].name, "Low"); // 1 use
    }

    #[test]
    fn test_bookmark_manager_record_use() {
        let mut manager = BookmarkManager::new();

        let id = manager
            .add_bookmark("Test", "echo test", None, vec![], None::<String>)
            .unwrap();

        let bookmark = manager.get_bookmark(id).unwrap();
        assert_eq!(bookmark.use_count, 0);

        manager.record_use(id).unwrap();

        let bookmark = manager.get_bookmark(id).unwrap();
        assert_eq!(bookmark.use_count, 1);
        assert!(bookmark.last_used.is_some());
    }

    #[test]
    fn test_bookmark_manager_clear() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark("Test1", "cmd1", None, vec![], None::<String>)
            .unwrap();
        manager
            .add_bookmark("Test2", "cmd2", None, vec![], None::<String>)
            .unwrap();

        assert_eq!(manager.len(), 2);

        manager.clear();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_file_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create and save bookmarks
        let mut manager = BookmarkManager::new();
        manager
            .add_bookmark(
                "Test1",
                "echo test1",
                Some(PathBuf::from("/tmp")),
                vec!["test".to_string()],
                Some("Description 1"),
            )
            .unwrap();
        manager
            .add_bookmark("Test2", "echo test2", None, vec![], None::<String>)
            .unwrap();

        manager.save_to_file(path.clone()).unwrap();

        // Load into new manager
        let mut new_manager = BookmarkManager::new();
        new_manager.load_from_file(path).unwrap();

        assert_eq!(new_manager.len(), 2);

        let bookmarks = new_manager.list_bookmarks();
        assert_eq!(bookmarks[0].name, "Test1");
        assert_eq!(bookmarks[0].command, "echo test1");
        assert_eq!(
            bookmarks[0].working_dir,
            Some(PathBuf::from("/tmp"))
        );
        assert_eq!(bookmarks[0].description, Some("Description 1".to_string()));
    }

    #[test]
    fn test_file_persistence_preserves_metadata() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let mut manager = BookmarkManager::new();
        let id = manager
            .add_bookmark("Test", "echo test", None, vec![], None::<String>)
            .unwrap();

        // Record some uses
        manager.record_use(id).unwrap();
        manager.record_use(id).unwrap();

        manager.save_to_file(path.clone()).unwrap();

        // Load and verify metadata is preserved
        let mut new_manager = BookmarkManager::new();
        new_manager.load_from_file(path).unwrap();

        let bookmarks = new_manager.list_bookmarks();
        assert_eq!(bookmarks[0].use_count, 2);
        assert!(bookmarks[0].last_used.is_some());
        assert!(bookmarks[0].created_at <= Utc::now());
    }

    #[test]
    fn test_default_bookmarks() {
        let manager = BookmarkManager::with_defaults();

        assert!(!manager.is_empty());

        // Check for some expected bookmarks
        let git_bookmarks = manager.get_by_tag("git");
        assert!(!git_bookmarks.is_empty());

        let docker_bookmarks = manager.get_by_tag("docker");
        assert!(!docker_bookmarks.is_empty());

        let rust_bookmarks = manager.get_by_tag("rust");
        assert!(!rust_bookmarks.is_empty());

        // Test search functionality
        let git_status = manager.search_bookmarks("Git Status");
        assert!(!git_status.is_empty());
    }

    #[test]
    fn test_bookmark_update_tags() {
        let mut manager = BookmarkManager::new();

        let id = manager
            .add_bookmark(
                "Test",
                "echo test",
                None,
                vec!["old".to_string()],
                None::<String>,
            )
            .unwrap();

        // Verify old tag works
        let old_results = manager.get_by_tag("old");
        assert_eq!(old_results.len(), 1);

        // Update tags
        let update = BookmarkUpdate::new().tags(vec!["new".to_string()]);
        manager.update_bookmark(id, update).unwrap();

        // Verify old tag no longer works
        let old_results = manager.get_by_tag("old");
        assert_eq!(old_results.len(), 0);

        // Verify new tag works
        let new_results = manager.get_by_tag("new");
        assert_eq!(new_results.len(), 1);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let mut manager = BookmarkManager::new();
        let result = manager.load_from_file(path.clone());

        assert!(result.is_ok());
        assert!(manager.is_empty());
        assert!(path.exists()); // Should create empty file
    }

    #[test]
    fn test_save_without_path() {
        let manager = BookmarkManager::new();
        let result = manager.save();
        assert!(matches!(result, Err(BookmarkError::NoFilePath)));
    }

    #[test]
    fn test_save_with_configured_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let mut manager = BookmarkManager::new();
        manager.load_from_file(path.clone()).unwrap();

        manager
            .add_bookmark("Test", "echo test", None, vec![], None::<String>)
            .unwrap();

        let result = manager.save();
        assert!(result.is_ok());

        // Verify file was written
        let mut new_manager = BookmarkManager::new();
        new_manager.load_from_file(path).unwrap();
        assert_eq!(new_manager.len(), 1);
    }
}
