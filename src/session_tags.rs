//! Session tagging system for AgTerm
//!
//! Provides comprehensive session organization with:
//! - Tag creation and management
//! - Session tagging and grouping
//! - Session pinning
//! - Notes and metadata
//! - Search and filtering

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Represents a tag that can be applied to sessions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    /// Tag name (unique identifier)
    pub name: String,
    /// Tag color as RGB tuple
    pub color: (u8, u8, u8),
    /// Optional icon (emoji or character)
    pub icon: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// When the tag was created
    pub created_at: DateTime<Utc>,
}

impl Tag {
    /// Create a new tag
    pub fn new(name: String, color: (u8, u8, u8)) -> Self {
        Self {
            name,
            color,
            icon: None,
            description: None,
            created_at: Utc::now(),
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Represents a tagged session with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionTag {
    /// Session identifier
    pub session_id: String,
    /// List of tag names applied to this session
    pub tags: Vec<String>,
    /// Optional notes about the session
    pub notes: Option<String>,
    /// Whether the session is pinned
    pub pinned: bool,
    /// When the session tag was created
    pub created_at: DateTime<Utc>,
    /// Last time the session was accessed
    pub last_accessed: DateTime<Utc>,
}

impl SessionTag {
    /// Create a new session tag
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            tags: Vec::new(),
            notes: None,
            pinned: false,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Check if session has a specific tag
    pub fn has_tag(&self, tag_name: &str) -> bool {
        self.tags.iter().any(|t| t == tag_name)
    }

    /// Add a tag if not already present
    pub fn add_tag(&mut self, tag_name: String) -> bool {
        if !self.has_tag(&tag_name) {
            self.tags.push(tag_name);
            true
        } else {
            false
        }
    }

    /// Remove a tag if present
    pub fn remove_tag(&mut self, tag_name: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag_name) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Updates that can be applied to a tag
#[derive(Debug, Clone)]
pub struct TagUpdate {
    /// New color (if specified)
    pub color: Option<(u8, u8, u8)>,
    /// New icon (if specified)
    pub icon: Option<Option<String>>,
    /// New description (if specified)
    pub description: Option<Option<String>>,
}

impl TagUpdate {
    /// Create an empty update
    pub fn new() -> Self {
        Self {
            color: None,
            icon: None,
            description: None,
        }
    }

    /// Set color
    pub fn with_color(mut self, color: (u8, u8, u8)) -> Self {
        self.color = Some(color);
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = Some(description);
        self
    }
}

impl Default for TagUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during tag operations
#[derive(Debug, Error)]
pub enum TagError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Tag already exists: {0}")]
    TagExists(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),
}

/// Manages tags available for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagManager {
    /// All available tags (keyed by name)
    tags: HashMap<String, Tag>,
}

impl TagManager {
    /// Create a new tag manager
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
        }
    }

    /// Create tag manager with default tags
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // Work tag
        let _ = manager.create_tag(
            "work".to_string(),
            (66, 133, 244), // Blue
            Some("💼".to_string()),
        );

        // Personal tag
        let _ = manager.create_tag(
            "personal".to_string(),
            (52, 168, 83), // Green
            Some("🏠".to_string()),
        );

        // Urgent tag
        let _ = manager.create_tag(
            "urgent".to_string(),
            (234, 67, 53), // Red
            Some("🔥".to_string()),
        );

        // Dev tag
        let _ = manager.create_tag(
            "dev".to_string(),
            (156, 39, 176), // Purple
            Some("💻".to_string()),
        );

        // Server tag
        let _ = manager.create_tag(
            "server".to_string(),
            (251, 140, 0), // Orange
            Some("🖥️".to_string()),
        );

        manager
    }

    /// Validate tag name
    fn validate_tag_name(name: &str) -> Result<(), TagError> {
        if name.is_empty() {
            return Err(TagError::InvalidTagName("Tag name cannot be empty".to_string()));
        }
        if name.contains(char::is_whitespace) {
            return Err(TagError::InvalidTagName("Tag name cannot contain whitespace".to_string()));
        }
        Ok(())
    }

    /// Create a new tag
    pub fn create_tag(
        &mut self,
        name: String,
        color: (u8, u8, u8),
        icon: Option<String>,
    ) -> Result<(), TagError> {
        Self::validate_tag_name(&name)?;

        if self.tags.contains_key(&name) {
            return Err(TagError::TagExists(name));
        }

        let mut tag = Tag::new(name.clone(), color);
        if let Some(icon_str) = icon {
            tag = tag.with_icon(icon_str);
        }

        self.tags.insert(name, tag);
        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&mut self, name: &str) -> Result<(), TagError> {
        self.tags
            .remove(name)
            .ok_or_else(|| TagError::TagNotFound(name.to_string()))?;
        Ok(())
    }

    /// Update a tag
    pub fn update_tag(&mut self, name: &str, updates: TagUpdate) -> Result<(), TagError> {
        let tag = self
            .tags
            .get_mut(name)
            .ok_or_else(|| TagError::TagNotFound(name.to_string()))?;

        if let Some(color) = updates.color {
            tag.color = color;
        }

        if let Some(icon) = updates.icon {
            tag.icon = icon;
        }

        if let Some(description) = updates.description {
            tag.description = description;
        }

        Ok(())
    }

    /// Get a tag by name
    pub fn get_tag(&self, name: &str) -> Option<&Tag> {
        self.tags.get(name)
    }

    /// List all tags
    pub fn list_tags(&self) -> Vec<&Tag> {
        let mut tags: Vec<&Tag> = self.tags.values().collect();
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        tags
    }

    /// Check if a tag exists
    pub fn has_tag(&self, name: &str) -> bool {
        self.tags.contains_key(name)
    }
}

impl Default for TagManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages session tagging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTagManager {
    /// Tag definitions
    tag_manager: TagManager,
    /// Session tags (keyed by session ID)
    sessions: HashMap<String, SessionTag>,
}

impl SessionTagManager {
    /// Create a new session tag manager
    pub fn new() -> Self {
        Self {
            tag_manager: TagManager::new(),
            sessions: HashMap::new(),
        }
    }

    /// Create with default tags
    pub fn with_defaults() -> Self {
        Self {
            tag_manager: TagManager::with_defaults(),
            sessions: HashMap::new(),
        }
    }

    /// Get or create a session tag entry
    fn get_or_create_session(&mut self, session_id: &str) -> &mut SessionTag {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionTag::new(session_id.to_string()))
    }

    /// Tag a session
    pub fn tag_session(&mut self, session_id: String, tag_name: String) -> Result<(), TagError> {
        // Verify tag exists
        if !self.tag_manager.has_tag(&tag_name) {
            return Err(TagError::TagNotFound(tag_name));
        }

        let session = self.get_or_create_session(&session_id);
        session.add_tag(tag_name);
        session.touch();

        Ok(())
    }

    /// Remove a tag from a session
    pub fn untag_session(&mut self, session_id: &str, tag_name: &str) -> Result<(), TagError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| TagError::SessionNotFound(session_id.to_string()))?;

        if !session.remove_tag(tag_name) {
            return Err(TagError::TagNotFound(tag_name.to_string()));
        }

        session.touch();
        Ok(())
    }

    /// Get all tags for a session
    pub fn get_session_tags(&self, session_id: &str) -> Vec<&Tag> {
        if let Some(session) = self.sessions.get(session_id) {
            session
                .tags
                .iter()
                .filter_map(|tag_name| self.tag_manager.get_tag(tag_name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all sessions with a specific tag
    pub fn get_sessions_by_tag(&self, tag_name: &str) -> Vec<&SessionTag> {
        self.sessions
            .values()
            .filter(|session| session.has_tag(tag_name))
            .collect()
    }

    /// Set notes for a session
    pub fn set_session_note(&mut self, session_id: String, note: Option<String>) -> Result<(), TagError> {
        let session = self.get_or_create_session(&session_id);
        session.notes = note;
        session.touch();
        Ok(())
    }

    /// Pin a session
    pub fn pin_session(&mut self, session_id: String) -> Result<(), TagError> {
        let session = self.get_or_create_session(&session_id);
        session.pinned = true;
        session.touch();
        Ok(())
    }

    /// Unpin a session
    pub fn unpin_session(&mut self, session_id: &str) -> Result<(), TagError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| TagError::SessionNotFound(session_id.to_string()))?;
        session.pinned = false;
        session.touch();
        Ok(())
    }

    /// Get all pinned sessions
    pub fn get_pinned_sessions(&self) -> Vec<&SessionTag> {
        let mut pinned: Vec<&SessionTag> = self
            .sessions
            .values()
            .filter(|session| session.pinned)
            .collect();

        // Sort by last accessed (most recent first)
        pinned.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        pinned
    }

    /// Search sessions by query (matches session ID, tags, or notes)
    pub fn search_sessions(&self, query: &str) -> Vec<&SessionTag> {
        let query_lower = query.to_lowercase();

        self.sessions
            .values()
            .filter(|session| {
                // Match session ID
                if session.session_id.to_lowercase().contains(&query_lower) {
                    return true;
                }

                // Match tags
                if session.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                    return true;
                }

                // Match notes
                if let Some(notes) = &session.notes {
                    if notes.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }

                false
            })
            .collect()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&SessionTag> {
        self.sessions.get(session_id)
    }

    /// Remove a session completely
    pub fn remove_session(&mut self, session_id: &str) -> Result<(), TagError> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| TagError::SessionNotFound(session_id.to_string()))?;
        Ok(())
    }

    /// Get tag manager (for tag operations)
    pub fn tag_manager(&self) -> &TagManager {
        &self.tag_manager
    }

    /// Get mutable tag manager
    pub fn tag_manager_mut(&mut self) -> &mut TagManager {
        &mut self.tag_manager
    }

    /// Save to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), TagError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to JSON with pretty formatting
        let json = serde_json::to_string_pretty(self)?;

        // Write to temporary file first, then rename (atomic operation)
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, json)?;
        std::fs::rename(&temp_path, path)?;

        tracing::info!("Session tags saved to {:?}", path);
        Ok(())
    }

    /// Load from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, TagError> {
        if !path.exists() {
            // Return default if file doesn't exist
            return Ok(Self::with_defaults());
        }

        let json = std::fs::read_to_string(path)?;
        let manager: SessionTagManager = serde_json::from_str(&json)?;

        tracing::info!("Session tags loaded from {:?}", path);
        Ok(manager)
    }

    /// Get default storage path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("session_tags.json")
    }
}

impl Default for SessionTagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tag_creation() {
        let mut manager = TagManager::new();

        // Create a tag
        let result = manager.create_tag(
            "test".to_string(),
            (255, 0, 0),
            Some("🔥".to_string()),
        );
        assert!(result.is_ok());

        // Verify tag exists
        let tag = manager.get_tag("test");
        assert!(tag.is_some());
        let tag = tag.unwrap();
        assert_eq!(tag.name, "test");
        assert_eq!(tag.color, (255, 0, 0));
        assert_eq!(tag.icon, Some("🔥".to_string()));

        // Duplicate tag should fail
        let result = manager.create_tag(
            "test".to_string(),
            (0, 255, 0),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_validation() {
        let mut manager = TagManager::new();

        // Empty name should fail
        let result = manager.create_tag("".to_string(), (0, 0, 0), None);
        assert!(result.is_err());

        // Name with whitespace should fail
        let result = manager.create_tag("test tag".to_string(), (0, 0, 0), None);
        assert!(result.is_err());

        // Valid name should succeed
        let result = manager.create_tag("valid_tag".to_string(), (0, 0, 0), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tag_update() {
        let mut manager = TagManager::new();
        manager.create_tag("test".to_string(), (255, 0, 0), None).unwrap();

        // Update color
        let update = TagUpdate::new().with_color((0, 255, 0));
        manager.update_tag("test", update).unwrap();

        let tag = manager.get_tag("test").unwrap();
        assert_eq!(tag.color, (0, 255, 0));

        // Update icon
        let update = TagUpdate::new().with_icon(Some("🎉".to_string()));
        manager.update_tag("test", update).unwrap();

        let tag = manager.get_tag("test").unwrap();
        assert_eq!(tag.icon, Some("🎉".to_string()));
    }

    #[test]
    fn test_tag_deletion() {
        let mut manager = TagManager::new();
        manager.create_tag("test".to_string(), (255, 0, 0), None).unwrap();

        // Delete tag
        let result = manager.delete_tag("test");
        assert!(result.is_ok());

        // Verify tag is gone
        assert!(manager.get_tag("test").is_none());

        // Deleting non-existent tag should fail
        let result = manager.delete_tag("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_tags() {
        let manager = TagManager::with_defaults();

        // Verify default tags exist
        assert!(manager.get_tag("work").is_some());
        assert!(manager.get_tag("personal").is_some());
        assert!(manager.get_tag("urgent").is_some());
        assert!(manager.get_tag("dev").is_some());
        assert!(manager.get_tag("server").is_some());

        // Verify tag count
        let tags = manager.list_tags();
        assert_eq!(tags.len(), 5);
    }

    #[test]
    fn test_session_tagging() {
        let mut manager = SessionTagManager::with_defaults();

        // Tag a session
        let result = manager.tag_session(
            "session1".to_string(),
            "work".to_string(),
        );
        assert!(result.is_ok());

        // Verify tag is applied
        let tags = manager.get_session_tags("session1");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "work");

        // Tag with non-existent tag should fail
        let result = manager.tag_session(
            "session1".to_string(),
            "nonexistent".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_session_untagging() {
        let mut manager = SessionTagManager::with_defaults();

        // Tag a session
        manager.tag_session("session1".to_string(), "work".to_string()).unwrap();

        // Untag the session
        let result = manager.untag_session("session1", "work");
        assert!(result.is_ok());

        // Verify tag is removed
        let tags = manager.get_session_tags("session1");
        assert_eq!(tags.len(), 0);

        // Untagging again should fail
        let result = manager.untag_session("session1", "work");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_sessions_by_tag() {
        let mut manager = SessionTagManager::with_defaults();

        // Tag multiple sessions with "work"
        manager.tag_session("session1".to_string(), "work".to_string()).unwrap();
        manager.tag_session("session2".to_string(), "work".to_string()).unwrap();
        manager.tag_session("session3".to_string(), "personal".to_string()).unwrap();

        // Get all work sessions
        let work_sessions = manager.get_sessions_by_tag("work");
        assert_eq!(work_sessions.len(), 2);

        // Get all personal sessions
        let personal_sessions = manager.get_sessions_by_tag("personal");
        assert_eq!(personal_sessions.len(), 1);
    }

    #[test]
    fn test_session_notes() {
        let mut manager = SessionTagManager::with_defaults();

        // Set notes
        let result = manager.set_session_note(
            "session1".to_string(),
            Some("Important session".to_string()),
        );
        assert!(result.is_ok());

        // Verify notes
        let session = manager.get_session("session1");
        assert!(session.is_some());
        assert_eq!(session.unwrap().notes, Some("Important session".to_string()));

        // Clear notes
        manager.set_session_note("session1".to_string(), None).unwrap();
        let session = manager.get_session("session1").unwrap();
        assert!(session.notes.is_none());
    }

    #[test]
    fn test_session_pinning() {
        let mut manager = SessionTagManager::with_defaults();

        // Pin a session
        manager.pin_session("session1".to_string()).unwrap();

        // Verify session is pinned
        let session = manager.get_session("session1").unwrap();
        assert!(session.pinned);

        // Check pinned sessions list
        let pinned = manager.get_pinned_sessions();
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].session_id, "session1");

        // Unpin the session
        manager.unpin_session("session1").unwrap();
        let session = manager.get_session("session1").unwrap();
        assert!(!session.pinned);
    }

    #[test]
    fn test_session_search() {
        let mut manager = SessionTagManager::with_defaults();

        // Create sessions with different attributes
        manager.tag_session("work_session".to_string(), "work".to_string()).unwrap();
        manager.tag_session("dev_session".to_string(), "dev".to_string()).unwrap();
        manager.set_session_note(
            "test_session".to_string(),
            Some("This is a test".to_string()),
        ).unwrap();

        // Search by session ID
        let results = manager.search_sessions("work");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "work_session");

        // Search by tag
        let results = manager.search_sessions("dev");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "dev_session");

        // Search by notes
        let results = manager.search_sessions("test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "test_session");

        // Search with no matches
        let results = manager.search_sessions("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("session_tags.json");

        // Create manager with some data
        let mut manager = SessionTagManager::with_defaults();
        manager.tag_session("session1".to_string(), "work".to_string()).unwrap();
        manager.set_session_note(
            "session1".to_string(),
            Some("Test note".to_string()),
        ).unwrap();
        manager.pin_session("session1".to_string()).unwrap();

        // Save to file
        manager.save_to_file(&file_path).unwrap();
        assert!(file_path.exists());

        // Load from file
        let loaded = SessionTagManager::load_from_file(&file_path).unwrap();

        // Verify data is preserved
        let session = loaded.get_session("session1").unwrap();
        assert_eq!(session.tags.len(), 1);
        assert_eq!(session.tags[0], "work");
        assert_eq!(session.notes, Some("Test note".to_string()));
        assert!(session.pinned);

        // Verify tags are preserved
        assert!(loaded.tag_manager().get_tag("work").is_some());
    }

    #[test]
    fn test_session_tag_methods() {
        let mut session_tag = SessionTag::new("test_session".to_string());

        // Test add_tag
        assert!(session_tag.add_tag("work".to_string()));
        assert!(!session_tag.add_tag("work".to_string())); // Duplicate should return false

        // Test has_tag
        assert!(session_tag.has_tag("work"));
        assert!(!session_tag.has_tag("personal"));

        // Test remove_tag
        assert!(session_tag.remove_tag("work"));
        assert!(!session_tag.remove_tag("work")); // Removing again should return false
        assert!(!session_tag.has_tag("work"));
    }

    #[test]
    fn test_session_touch() {
        let mut session_tag = SessionTag::new("test_session".to_string());
        let initial_time = session_tag.last_accessed;

        // Wait a bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Touch the session
        session_tag.touch();

        // Verify last_accessed was updated
        assert!(session_tag.last_accessed > initial_time);
    }

    #[test]
    fn test_remove_session() {
        let mut manager = SessionTagManager::with_defaults();

        // Create a session
        manager.tag_session("session1".to_string(), "work".to_string()).unwrap();
        assert!(manager.get_session("session1").is_some());

        // Remove the session
        let result = manager.remove_session("session1");
        assert!(result.is_ok());
        assert!(manager.get_session("session1").is_none());

        // Removing again should fail
        let result = manager.remove_session("session1");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        // Loading from non-existent file should return defaults
        let manager = SessionTagManager::load_from_file(&file_path).unwrap();

        // Should have default tags
        assert!(manager.tag_manager().get_tag("work").is_some());
        assert!(manager.tag_manager().get_tag("personal").is_some());
    }
}
