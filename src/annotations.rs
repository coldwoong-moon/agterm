//! Terminal annotation system for marking and bookmarking terminal lines
//!
//! Provides functionality to:
//! - Add notes, warnings, and bookmarks to terminal lines
//! - Persist annotations to disk
//! - Search and navigate through annotations
//! - Color-coded visual markers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Type of annotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnnotationType {
    /// Regular note
    Note,
    /// Warning or alert
    Warning,
    /// Bookmark for quick navigation
    Bookmark,
}

impl AnnotationType {
    /// Get default color for this annotation type
    pub fn default_color(&self) -> [u8; 3] {
        match self {
            AnnotationType::Note => [100, 149, 237],      // Cornflower blue
            AnnotationType::Warning => [255, 165, 0],     // Orange
            AnnotationType::Bookmark => [50, 205, 50],    // Lime green
        }
    }

    /// Get display symbol for this annotation type
    pub fn symbol(&self) -> &'static str {
        match self {
            AnnotationType::Note => "📝",
            AnnotationType::Warning => "⚠️",
            AnnotationType::Bookmark => "🔖",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            AnnotationType::Note => "Note",
            AnnotationType::Warning => "Warning",
            AnnotationType::Bookmark => "Bookmark",
        }
    }
}

/// Range of lines (inclusive)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    /// Create a single-line range
    pub fn single(line: usize) -> Self {
        Self {
            start: line,
            end: line,
        }
    }

    /// Create a multi-line range
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    /// Check if this range contains a line
    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &LineRange) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    /// Get the number of lines in this range
    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }

    /// Check if this is a single-line range
    pub fn is_single_line(&self) -> bool {
        self.start == self.end
    }
}

/// A single annotation on terminal lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Unique identifier
    pub id: String,
    /// Lines this annotation applies to
    pub range: LineRange,
    /// Annotation content
    pub content: String,
    /// Type of annotation
    pub annotation_type: AnnotationType,
    /// Custom color (RGB), or None to use default
    pub color: Option<[u8; 3]>,
    /// When this annotation was created
    pub created_at: DateTime<Utc>,
    /// When this annotation was last modified
    pub modified_at: DateTime<Utc>,
    /// Optional tags for categorization
    pub tags: Vec<String>,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(range: LineRange, content: String, annotation_type: AnnotationType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            range,
            content,
            annotation_type,
            color: None,
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
        }
    }

    /// Create a note annotation
    pub fn note(line: usize, content: String) -> Self {
        Self::new(LineRange::single(line), content, AnnotationType::Note)
    }

    /// Create a warning annotation
    pub fn warning(line: usize, content: String) -> Self {
        Self::new(
            LineRange::single(line),
            content,
            AnnotationType::Warning,
        )
    }

    /// Create a bookmark annotation
    pub fn bookmark(line: usize, content: String) -> Self {
        Self::new(
            LineRange::single(line),
            content,
            AnnotationType::Bookmark,
        )
    }

    /// Update the content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.modified_at = Utc::now();
    }

    /// Set custom color
    pub fn set_color(&mut self, color: [u8; 3]) {
        self.color = Some(color);
        self.modified_at = Utc::now();
    }

    /// Get effective color (custom or default)
    pub fn effective_color(&self) -> [u8; 3] {
        self.color
            .unwrap_or_else(|| self.annotation_type.default_color())
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.modified_at = Utc::now();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.modified_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if annotation has a tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if this annotation applies to a line
    pub fn applies_to_line(&self, line: usize) -> bool {
        self.range.contains(line)
    }
}

/// Manages all annotations for a terminal session
#[derive(Debug, Clone)]
pub struct AnnotationManager {
    /// All annotations, indexed by ID
    annotations: HashMap<String, Annotation>,
    /// Line to annotation IDs mapping for fast lookup
    line_index: HashMap<usize, Vec<String>>,
    /// Path to persistence file
    file_path: Option<PathBuf>,
    /// Maximum number of annotations to keep
    max_annotations: usize,
}

impl AnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: HashMap::new(),
            line_index: HashMap::new(),
            file_path: None,
            max_annotations: 10000,
        }
    }

    /// Create with custom max annotations
    pub fn with_max_annotations(max_annotations: usize) -> Self {
        Self {
            annotations: HashMap::new(),
            line_index: HashMap::new(),
            file_path: None,
            max_annotations,
        }
    }

    /// Add an annotation
    pub fn add(&mut self, annotation: Annotation) -> String {
        let id = annotation.id.clone();

        // Update line index for all lines in range
        for line in annotation.range.start..=annotation.range.end {
            self.line_index
                .entry(line)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        self.annotations.insert(id.clone(), annotation);

        // Trim if needed
        self.trim_old_annotations();

        tracing::debug!("Added annotation {}", id);
        id
    }

    /// Remove an annotation by ID
    pub fn remove(&mut self, id: &str) -> Option<Annotation> {
        if let Some(annotation) = self.annotations.remove(id) {
            // Remove from line index
            for line in annotation.range.start..=annotation.range.end {
                if let Some(ids) = self.line_index.get_mut(&line) {
                    ids.retain(|i| i != id);
                    if ids.is_empty() {
                        self.line_index.remove(&line);
                    }
                }
            }
            tracing::debug!("Removed annotation {}", id);
            Some(annotation)
        } else {
            None
        }
    }

    /// Update an annotation's content
    pub fn update_content(&mut self, id: &str, content: String) -> bool {
        if let Some(annotation) = self.annotations.get_mut(id) {
            annotation.set_content(content);
            tracing::debug!("Updated annotation {} content", id);
            true
        } else {
            false
        }
    }

    /// Update an annotation's color
    pub fn update_color(&mut self, id: &str, color: [u8; 3]) -> bool {
        if let Some(annotation) = self.annotations.get_mut(id) {
            annotation.set_color(color);
            tracing::debug!("Updated annotation {} color", id);
            true
        } else {
            false
        }
    }

    /// Get an annotation by ID
    pub fn get(&self, id: &str) -> Option<&Annotation> {
        self.annotations.get(id)
    }

    /// Get all annotations for a specific line
    pub fn get_for_line(&self, line: usize) -> Vec<&Annotation> {
        self.line_index
            .get(&line)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.annotations.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all annotations of a specific type
    pub fn get_by_type(&self, annotation_type: AnnotationType) -> Vec<&Annotation> {
        self.annotations
            .values()
            .filter(|a| a.annotation_type == annotation_type)
            .collect()
    }

    /// Search annotations by content
    pub fn search(&self, query: &str) -> Vec<&Annotation> {
        let query_lower = query.to_lowercase();
        self.annotations
            .values()
            .filter(|a| a.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Search annotations by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Annotation> {
        self.annotations
            .values()
            .filter(|a| a.has_tag(tag))
            .collect()
    }

    /// Get all bookmarks sorted by line number
    pub fn get_bookmarks(&self) -> Vec<&Annotation> {
        let mut bookmarks = self.get_by_type(AnnotationType::Bookmark);
        bookmarks.sort_by_key(|a| a.range.start);
        bookmarks
    }

    /// Navigate to next bookmark after given line
    pub fn next_bookmark(&self, current_line: usize) -> Option<&Annotation> {
        self.get_bookmarks()
            .into_iter()
            .find(|a| a.range.start > current_line)
    }

    /// Navigate to previous bookmark before given line
    pub fn prev_bookmark(&self, current_line: usize) -> Option<&Annotation> {
        self.get_bookmarks()
            .into_iter()
            .rev()
            .find(|a| a.range.start < current_line)
    }

    /// Get all annotations sorted by line number
    pub fn all_sorted(&self) -> Vec<&Annotation> {
        let mut annotations: Vec<_> = self.annotations.values().collect();
        annotations.sort_by_key(|a| a.range.start);
        annotations
    }

    /// Get total count of annotations
    pub fn count(&self) -> usize {
        self.annotations.len()
    }

    /// Get count by type
    pub fn count_by_type(&self, annotation_type: AnnotationType) -> usize {
        self.annotations
            .values()
            .filter(|a| a.annotation_type == annotation_type)
            .count()
    }

    /// Check if a line has any annotations
    pub fn has_annotations_at_line(&self, line: usize) -> bool {
        self.line_index
            .get(&line)
            .map(|ids| !ids.is_empty())
            .unwrap_or(false)
    }

    /// Clear all annotations
    pub fn clear(&mut self) {
        self.annotations.clear();
        self.line_index.clear();
        tracing::info!("Cleared all annotations");
    }

    /// Clear annotations of a specific type
    pub fn clear_by_type(&mut self, annotation_type: AnnotationType) {
        let ids_to_remove: Vec<_> = self
            .annotations
            .values()
            .filter(|a| a.annotation_type == annotation_type)
            .map(|a| a.id.clone())
            .collect();

        for id in ids_to_remove {
            self.remove(&id);
        }

        tracing::info!("Cleared annotations of type {:?}", annotation_type);
    }

    /// Load annotations from file
    pub fn load_from_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Create empty file
            File::create(&path)?;
            self.file_path = Some(path);
            return Ok(());
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        self.clear();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if !line.is_empty() {
                match serde_json::from_str::<Annotation>(line) {
                    Ok(annotation) => {
                        let id = annotation.id.clone();
                        // Update line index
                        for line_num in annotation.range.start..=annotation.range.end {
                            self.line_index
                                .entry(line_num)
                                .or_insert_with(Vec::new)
                                .push(id.clone());
                        }
                        self.annotations.insert(id, annotation);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse annotation line: {}", e);
                    }
                }
            }
        }

        self.file_path = Some(path);
        tracing::info!("Loaded {} annotations", self.annotations.len());
        Ok(())
    }

    /// Set the file path for persistence
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Save annotations to file
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

            // Write all annotations as JSON lines
            for annotation in self.annotations.values() {
                let json = serde_json::to_string(annotation)?;
                writeln!(file, "{}", json)?;
            }

            tracing::debug!(
                "Saved {} annotations to {:?}",
                self.annotations.len(),
                path
            );
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Trim oldest annotations if we exceed max
    fn trim_old_annotations(&mut self) {
        if self.annotations.len() > self.max_annotations {
            let excess = self.annotations.len() - self.max_annotations;

            // Sort by creation time and remove oldest
            let mut sorted: Vec<_> = self.annotations.values().collect();
            sorted.sort_by_key(|a| a.created_at);

            let ids_to_remove: Vec<_> = sorted
                .iter()
                .take(excess)
                .map(|a| a.id.clone())
                .collect();

            for id in ids_to_remove {
                self.remove(&id);
            }

            tracing::debug!("Trimmed {} old annotations", excess);
        }
    }

    /// Get statistics about annotations
    pub fn stats(&self) -> AnnotationStats {
        AnnotationStats {
            total: self.count(),
            notes: self.count_by_type(AnnotationType::Note),
            warnings: self.count_by_type(AnnotationType::Warning),
            bookmarks: self.count_by_type(AnnotationType::Bookmark),
        }
    }
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about annotations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnnotationStats {
    pub total: usize,
    pub notes: usize,
    pub warnings: usize,
    pub bookmarks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_annotation_creation() {
        let note = Annotation::note(10, "Test note".to_string());
        assert_eq!(note.range.start, 10);
        assert_eq!(note.range.end, 10);
        assert_eq!(note.annotation_type, AnnotationType::Note);
        assert_eq!(note.content, "Test note");
    }

    #[test]
    fn test_annotation_types() {
        let note = AnnotationType::Note;
        let warning = AnnotationType::Warning;
        let bookmark = AnnotationType::Bookmark;

        assert_eq!(note.name(), "Note");
        assert_eq!(warning.name(), "Warning");
        assert_eq!(bookmark.name(), "Bookmark");

        assert_eq!(note.symbol(), "📝");
        assert_eq!(warning.symbol(), "⚠️");
        assert_eq!(bookmark.symbol(), "🔖");
    }

    #[test]
    fn test_line_range() {
        let single = LineRange::single(5);
        assert!(single.is_single_line());
        assert_eq!(single.len(), 1);
        assert!(single.contains(5));
        assert!(!single.contains(4));
        assert!(!single.contains(6));

        let range = LineRange::new(10, 15);
        assert!(!range.is_single_line());
        assert_eq!(range.len(), 6);
        assert!(range.contains(10));
        assert!(range.contains(12));
        assert!(range.contains(15));
        assert!(!range.contains(9));
        assert!(!range.contains(16));
    }

    #[test]
    fn test_range_overlap() {
        let range1 = LineRange::new(5, 10);
        let range2 = LineRange::new(8, 12);
        let range3 = LineRange::new(11, 15);

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
        assert!(!range1.overlaps(&range3));
        assert!(!range3.overlaps(&range1));
        assert!(range2.overlaps(&range3));
    }

    #[test]
    fn test_annotation_tags() {
        let mut annotation = Annotation::note(5, "Test".to_string());

        annotation.add_tag("important".to_string());
        annotation.add_tag("review".to_string());

        assert!(annotation.has_tag("important"));
        assert!(annotation.has_tag("review"));
        assert!(!annotation.has_tag("other"));

        assert_eq!(annotation.tags.len(), 2);

        // Adding duplicate should not increase count
        annotation.add_tag("important".to_string());
        assert_eq!(annotation.tags.len(), 2);

        // Remove tag
        assert!(annotation.remove_tag("important"));
        assert!(!annotation.has_tag("important"));
        assert_eq!(annotation.tags.len(), 1);

        // Remove non-existent tag
        assert!(!annotation.remove_tag("nonexistent"));
    }

    #[test]
    fn test_annotation_color() {
        let mut annotation = Annotation::note(5, "Test".to_string());

        // Should use default color initially
        let default_color = AnnotationType::Note.default_color();
        assert_eq!(annotation.effective_color(), default_color);

        // Set custom color
        let custom_color = [255, 0, 0];
        annotation.set_color(custom_color);
        assert_eq!(annotation.effective_color(), custom_color);
    }

    #[test]
    fn test_manager_add_and_get() {
        let mut manager = AnnotationManager::new();

        let annotation = Annotation::note(10, "Test note".to_string());
        let id = manager.add(annotation);

        assert_eq!(manager.count(), 1);

        let retrieved = manager.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test note");
    }

    #[test]
    fn test_manager_get_for_line() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Note 1".to_string()));
        manager.add(Annotation::warning(10, "Warning 1".to_string()));
        manager.add(Annotation::bookmark(11, "Bookmark 1".to_string()));

        let line10_annotations = manager.get_for_line(10);
        assert_eq!(line10_annotations.len(), 2);

        let line11_annotations = manager.get_for_line(11);
        assert_eq!(line11_annotations.len(), 1);

        let line12_annotations = manager.get_for_line(12);
        assert_eq!(line12_annotations.len(), 0);
    }

    #[test]
    fn test_manager_multiline_annotation() {
        let mut manager = AnnotationManager::new();

        let annotation = Annotation::new(
            LineRange::new(10, 15),
            "Multi-line note".to_string(),
            AnnotationType::Note,
        );
        manager.add(annotation);

        for line in 10..=15 {
            assert!(manager.has_annotations_at_line(line));
            let annotations = manager.get_for_line(line);
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0].content, "Multi-line note");
        }

        assert!(!manager.has_annotations_at_line(9));
        assert!(!manager.has_annotations_at_line(16));
    }

    #[test]
    fn test_manager_remove() {
        let mut manager = AnnotationManager::new();

        let annotation = Annotation::note(10, "Test".to_string());
        let id = manager.add(annotation);

        assert_eq!(manager.count(), 1);
        assert!(manager.has_annotations_at_line(10));

        let removed = manager.remove(&id);
        assert!(removed.is_some());
        assert_eq!(manager.count(), 0);
        assert!(!manager.has_annotations_at_line(10));
    }

    #[test]
    fn test_manager_update() {
        let mut manager = AnnotationManager::new();

        let annotation = Annotation::note(10, "Original".to_string());
        let id = manager.add(annotation);

        assert!(manager.update_content(&id, "Updated".to_string()));

        let updated = manager.get(&id);
        assert_eq!(updated.unwrap().content, "Updated");
    }

    #[test]
    fn test_manager_search() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Git commit".to_string()));
        manager.add(Annotation::note(11, "Git push".to_string()));
        manager.add(Annotation::note(12, "Docker build".to_string()));

        let results = manager.search("git");
        assert_eq!(results.len(), 2);

        let results = manager.search("docker");
        assert_eq!(results.len(), 1);

        let results = manager.search("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_manager_search_case_insensitive() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Test Message".to_string()));

        let results = manager.search("test");
        assert_eq!(results.len(), 1);

        let results = manager.search("MESSAGE");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_manager_by_type() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Note 1".to_string()));
        manager.add(Annotation::note(11, "Note 2".to_string()));
        manager.add(Annotation::warning(12, "Warning 1".to_string()));
        manager.add(Annotation::bookmark(13, "Bookmark 1".to_string()));

        let notes = manager.get_by_type(AnnotationType::Note);
        assert_eq!(notes.len(), 2);

        let warnings = manager.get_by_type(AnnotationType::Warning);
        assert_eq!(warnings.len(), 1);

        let bookmarks = manager.get_by_type(AnnotationType::Bookmark);
        assert_eq!(bookmarks.len(), 1);
    }

    #[test]
    fn test_bookmark_navigation() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::bookmark(10, "Bookmark 1".to_string()));
        manager.add(Annotation::bookmark(20, "Bookmark 2".to_string()));
        manager.add(Annotation::bookmark(30, "Bookmark 3".to_string()));

        let next = manager.next_bookmark(5);
        assert_eq!(next.unwrap().range.start, 10);

        let next = manager.next_bookmark(10);
        assert_eq!(next.unwrap().range.start, 20);

        let next = manager.next_bookmark(30);
        assert!(next.is_none());

        let prev = manager.prev_bookmark(35);
        assert_eq!(prev.unwrap().range.start, 30);

        let prev = manager.prev_bookmark(20);
        assert_eq!(prev.unwrap().range.start, 10);

        let prev = manager.prev_bookmark(10);
        assert!(prev.is_none());
    }

    #[test]
    fn test_manager_clear() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Note".to_string()));
        manager.add(Annotation::warning(11, "Warning".to_string()));

        assert_eq!(manager.count(), 2);

        manager.clear();
        assert_eq!(manager.count(), 0);
        assert!(!manager.has_annotations_at_line(10));
        assert!(!manager.has_annotations_at_line(11));
    }

    #[test]
    fn test_manager_clear_by_type() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Note 1".to_string()));
        manager.add(Annotation::note(11, "Note 2".to_string()));
        manager.add(Annotation::warning(12, "Warning".to_string()));

        assert_eq!(manager.count(), 3);

        manager.clear_by_type(AnnotationType::Note);
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.count_by_type(AnnotationType::Note), 0);
        assert_eq!(manager.count_by_type(AnnotationType::Warning), 1);
    }

    #[test]
    fn test_manager_stats() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(10, "Note 1".to_string()));
        manager.add(Annotation::note(11, "Note 2".to_string()));
        manager.add(Annotation::warning(12, "Warning".to_string()));
        manager.add(Annotation::bookmark(13, "Bookmark".to_string()));

        let stats = manager.stats();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.notes, 2);
        assert_eq!(stats.warnings, 1);
        assert_eq!(stats.bookmarks, 1);
    }

    #[test]
    fn test_manager_sorted() {
        let mut manager = AnnotationManager::new();

        manager.add(Annotation::note(30, "Third".to_string()));
        manager.add(Annotation::note(10, "First".to_string()));
        manager.add(Annotation::note(20, "Second".to_string()));

        let sorted = manager.all_sorted();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].range.start, 10);
        assert_eq!(sorted[1].range.start, 20);
        assert_eq!(sorted[2].range.start, 30);
    }

    #[test]
    fn test_manager_search_by_tag() {
        let mut manager = AnnotationManager::new();

        let mut annotation1 = Annotation::note(10, "Note 1".to_string());
        annotation1.add_tag("important".to_string());
        manager.add(annotation1);

        let mut annotation2 = Annotation::note(11, "Note 2".to_string());
        annotation2.add_tag("important".to_string());
        annotation2.add_tag("review".to_string());
        manager.add(annotation2);

        let mut annotation3 = Annotation::note(12, "Note 3".to_string());
        annotation3.add_tag("review".to_string());
        manager.add(annotation3);

        let important = manager.search_by_tag("important");
        assert_eq!(important.len(), 2);

        let review = manager.search_by_tag("review");
        assert_eq!(review.len(), 2);

        let other = manager.search_by_tag("other");
        assert_eq!(other.len(), 0);
    }

    #[test]
    fn test_file_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create and save annotations
        let mut manager = AnnotationManager::new();
        manager.add(Annotation::note(10, "Note 1".to_string()));
        manager.add(Annotation::warning(20, "Warning 1".to_string()));
        manager.add(Annotation::bookmark(30, "Bookmark 1".to_string()));

        manager.set_file_path(path.clone());
        manager.save_to_file().unwrap();

        // Load into new manager
        let mut manager2 = AnnotationManager::new();
        manager2.load_from_file(path).unwrap();

        assert_eq!(manager2.count(), 3);
        assert_eq!(manager2.count_by_type(AnnotationType::Note), 1);
        assert_eq!(manager2.count_by_type(AnnotationType::Warning), 1);
        assert_eq!(manager2.count_by_type(AnnotationType::Bookmark), 1);

        assert!(manager2.has_annotations_at_line(10));
        assert!(manager2.has_annotations_at_line(20));
        assert!(manager2.has_annotations_at_line(30));
    }

    #[test]
    fn test_max_annotations_trim() {
        let mut manager = AnnotationManager::with_max_annotations(3);

        manager.add(Annotation::note(10, "Note 1".to_string()));
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.add(Annotation::note(11, "Note 2".to_string()));
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.add(Annotation::note(12, "Note 3".to_string()));
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(manager.count(), 3);

        // Adding a 4th should remove the oldest
        manager.add(Annotation::note(13, "Note 4".to_string()));
        assert_eq!(manager.count(), 3);

        // The oldest (line 10) should be gone
        assert!(!manager.has_annotations_at_line(10));
        assert!(manager.has_annotations_at_line(11));
        assert!(manager.has_annotations_at_line(12));
        assert!(manager.has_annotations_at_line(13));
    }
}
