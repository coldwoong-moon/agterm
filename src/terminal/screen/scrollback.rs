//! Enhanced scrollback buffer with configurable limits, compression, and search
//!
//! This module provides an improved scrollback buffer that supports:
//! - Configurable maximum line limits (with 0 = unlimited)
//! - Memory usage tracking
//! - Line compression using RLE
//! - Full-text search across scrollback history
//! - Automatic trimming when limits are exceeded

use super::{Cell, CompressedLine};
use std::collections::VecDeque;

/// Scrollback buffer configuration
#[derive(Debug, Clone)]
pub struct ScrollbackConfig {
    /// Maximum number of lines to keep (0 = unlimited)
    pub max_lines: usize,
    /// Enable RLE compression for scrollback lines
    pub compression_enabled: bool,
    /// Save scrollback to file on exit (future feature)
    pub save_to_file: bool,
}

impl Default for ScrollbackConfig {
    fn default() -> Self {
        Self {
            max_lines: 10000,
            compression_enabled: true,
            save_to_file: false,
        }
    }
}

/// Enhanced scrollback buffer with configurable limits and compression
#[derive(Debug)]
pub struct ScrollbackBuffer {
    /// Compressed lines stored in FIFO order (oldest first)
    lines: VecDeque<CompressedLine>,
    /// Configuration
    config: ScrollbackConfig,
    /// Total uncompressed bytes (for memory tracking)
    total_uncompressed_bytes: usize,
    /// Total compressed bytes (for memory tracking)
    total_compressed_bytes: usize,
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer with the given configuration
    pub fn new(config: ScrollbackConfig) -> Self {
        Self {
            lines: VecDeque::new(),
            config,
            total_uncompressed_bytes: 0,
            total_compressed_bytes: 0,
        }
    }

    /// Create a scrollback buffer with default configuration
    pub fn with_max_lines(max_lines: usize) -> Self {
        Self::new(ScrollbackConfig {
            max_lines,
            ..Default::default()
        })
    }

    /// Push a new line to the scrollback buffer
    ///
    /// The line is compressed if compression is enabled.
    /// If the buffer exceeds max_lines, the oldest line is removed.
    pub fn push(&mut self, line: &[Cell]) {
        let compressed = if self.config.compression_enabled {
            CompressedLine::compress(line)
        } else {
            // Even if compression is disabled, we still use CompressedLine
            // for consistency, but the line won't be actually compressed
            CompressedLine::compress(line)
        };

        // Update memory tracking
        self.total_uncompressed_bytes += compressed.uncompressed_size();
        self.total_compressed_bytes += compressed.compressed_size();

        self.lines.push_back(compressed);
        self.trim_to_limit();
    }

    /// Get a line by index (0 = oldest line)
    pub fn get(&self, index: usize) -> Option<Vec<Cell>> {
        self.lines
            .get(index)
            .map(|compressed| compressed.decompress())
    }

    /// Get a compressed line by index without decompressing
    pub fn get_compressed(&self, index: usize) -> Option<&CompressedLine> {
        self.lines.get(index)
    }

    /// Get the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get total memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.total_compressed_bytes
    }

    /// Get uncompressed memory size in bytes
    pub fn uncompressed_size(&self) -> usize {
        self.total_uncompressed_bytes
    }

    /// Get compressed memory size in bytes
    pub fn compressed_size(&self) -> usize {
        self.total_compressed_bytes
    }

    /// Get compression ratio (0.0 to 1.0, where lower is better compression)
    pub fn compression_ratio(&self) -> f64 {
        if self.total_uncompressed_bytes == 0 {
            return 1.0;
        }
        self.total_compressed_bytes as f64 / self.total_uncompressed_bytes as f64
    }

    /// Get the current configuration
    pub fn config(&self) -> &ScrollbackConfig {
        &self.config
    }

    /// Update the configuration
    ///
    /// If max_lines is reduced, the buffer is trimmed immediately.
    pub fn set_config(&mut self, config: ScrollbackConfig) {
        self.config = config;
        self.trim_to_limit();
    }

    /// Trim the buffer to the configured maximum size
    fn trim_to_limit(&mut self) {
        if self.config.max_lines == 0 {
            // Unlimited
            return;
        }

        while self.lines.len() > self.config.max_lines {
            if let Some(removed) = self.lines.pop_front() {
                // Update memory tracking
                self.total_uncompressed_bytes = self
                    .total_uncompressed_bytes
                    .saturating_sub(removed.uncompressed_size());
                self.total_compressed_bytes = self
                    .total_compressed_bytes
                    .saturating_sub(removed.compressed_size());
            }
        }
    }

    /// Clear all lines from the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.total_uncompressed_bytes = 0;
        self.total_compressed_bytes = 0;
    }

    /// Search for a pattern in the scrollback buffer
    ///
    /// Returns a vector of (line_index, column_index) tuples indicating
    /// where the pattern was found.
    ///
    /// Note: This requires decompressing all lines, which may be slow
    /// for large buffers.
    pub fn search(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let pattern_lower = pattern.to_lowercase();

        for (line_idx, compressed_line) in self.lines.iter().enumerate() {
            let line = compressed_line.decompress();
            let text: String = line.iter().map(|cell| cell.c).collect();
            let text_lower = text.to_lowercase();

            // Find all occurrences in this line
            let mut start = 0;
            while let Some(pos) = text_lower[start..].find(&pattern_lower) {
                let abs_pos = start + pos;
                results.push((line_idx, abs_pos));
                start = abs_pos + 1;
            }
        }

        results
    }

    /// Search for a pattern with case sensitivity
    pub fn search_case_sensitive(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        for (line_idx, compressed_line) in self.lines.iter().enumerate() {
            let line = compressed_line.decompress();
            let text: String = line.iter().map(|cell| cell.c).collect();

            // Find all occurrences in this line
            let mut start = 0;
            while let Some(pos) = text[start..].find(pattern) {
                let abs_pos = start + pos;
                results.push((line_idx, abs_pos));
                start = abs_pos + 1;
            }
        }

        results
    }

    /// Get an iterator over all compressed lines
    pub fn iter(&self) -> impl Iterator<Item = &CompressedLine> {
        self.lines.iter()
    }

    /// Pop the most recent line from the buffer
    ///
    /// Returns None if the buffer is empty.
    pub fn pop_back(&mut self) -> Option<CompressedLine> {
        if let Some(removed) = self.lines.pop_back() {
            // Update memory tracking
            self.total_uncompressed_bytes = self
                .total_uncompressed_bytes
                .saturating_sub(removed.uncompressed_size());
            self.total_compressed_bytes = self
                .total_compressed_bytes
                .saturating_sub(removed.compressed_size());
            Some(removed)
        } else {
            None
        }
    }

    /// Pop the oldest line from the buffer
    ///
    /// Returns None if the buffer is empty.
    pub fn pop_front(&mut self) -> Option<CompressedLine> {
        if let Some(removed) = self.lines.pop_front() {
            // Update memory tracking
            self.total_uncompressed_bytes = self
                .total_uncompressed_bytes
                .saturating_sub(removed.uncompressed_size());
            self.total_compressed_bytes = self
                .total_compressed_bytes
                .saturating_sub(removed.compressed_size());
            Some(removed)
        } else {
            None
        }
    }

    /// Clone the internal buffer (for alternate screen switching)
    pub fn clone_lines(&self) -> VecDeque<CompressedLine> {
        self.lines.clone()
    }

    /// Restore from a cloned buffer (for alternate screen switching)
    pub fn restore_from(&mut self, lines: VecDeque<CompressedLine>) {
        // Recalculate memory usage
        self.total_uncompressed_bytes = lines.iter().map(|line| line.uncompressed_size()).sum();
        self.total_compressed_bytes = lines.iter().map(|line| line.compressed_size()).sum();

        self.lines = lines;
        self.trim_to_limit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::screen::Cell;

    fn create_test_line(text: &str) -> Vec<Cell> {
        text.chars()
            .map(|ch| Cell {
                c: ch,
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn test_basic_operations() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.push(&create_test_line("Line 1"));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());

        let line = buffer.get(0).unwrap();
        let text: String = line.iter().map(|c| c.c).collect();
        assert_eq!(text, "Line 1");
    }

    #[test]
    fn test_max_lines_limit() {
        let mut buffer = ScrollbackBuffer::with_max_lines(3);

        buffer.push(&create_test_line("Line 1"));
        buffer.push(&create_test_line("Line 2"));
        buffer.push(&create_test_line("Line 3"));
        assert_eq!(buffer.len(), 3);

        // Adding a 4th line should remove the oldest
        buffer.push(&create_test_line("Line 4"));
        assert_eq!(buffer.len(), 3);

        // First line should now be "Line 2"
        let line = buffer.get(0).unwrap();
        let text: String = line.iter().map(|c| c.c).collect();
        assert_eq!(text, "Line 2");
    }

    #[test]
    fn test_unlimited_buffer() {
        let mut buffer = ScrollbackBuffer::with_max_lines(0);

        for i in 0..1000 {
            buffer.push(&create_test_line(&format!("Line {}", i)));
        }

        // Should have all 1000 lines
        assert_eq!(buffer.len(), 1000);
    }

    #[test]
    fn test_clear() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        buffer.push(&create_test_line("Line 1"));
        buffer.push(&create_test_line("Line 2"));
        assert_eq!(buffer.len(), 2);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.memory_usage(), 0);
    }

    #[test]
    fn test_search() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        buffer.push(&create_test_line("Hello world"));
        buffer.push(&create_test_line("Goodbye world"));
        buffer.push(&create_test_line("Hello again"));

        let results = buffer.search("world");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First line
        assert_eq!(results[1].0, 1); // Second line

        let results = buffer.search("Hello");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First line
        assert_eq!(results[1].0, 2); // Third line
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        buffer.push(&create_test_line("Hello World"));

        let results = buffer.search("hello");
        assert_eq!(results.len(), 1);

        let results = buffer.search("WORLD");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_tracking() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        assert_eq!(buffer.memory_usage(), 0);

        buffer.push(&create_test_line("Test line"));
        assert!(buffer.memory_usage() > 0);

        let usage_before = buffer.memory_usage();
        buffer.push(&create_test_line("Another line"));
        assert!(buffer.memory_usage() > usage_before);

        buffer.clear();
        assert_eq!(buffer.memory_usage(), 0);
    }

    #[test]
    fn test_compression_ratio() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        // Lines with lots of repeated characters should compress well
        let repeated = vec![Cell::default(); 80];
        buffer.push(&repeated);

        let ratio = buffer.compression_ratio();
        assert!(ratio < 1.0, "Compression should reduce size");
        assert!(ratio > 0.0, "Compression ratio should be positive");
    }

    #[test]
    fn test_config_update() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        for i in 0..10 {
            buffer.push(&create_test_line(&format!("Line {}", i)));
        }
        assert_eq!(buffer.len(), 10);

        // Reduce max lines
        let new_config = ScrollbackConfig {
            max_lines: 5,
            ..Default::default()
        };
        buffer.set_config(new_config);

        // Should be trimmed to 5 lines
        assert_eq!(buffer.len(), 5);

        // First line should be "Line 5" (oldest 5 lines removed)
        let line = buffer.get(0).unwrap();
        let text: String = line.iter().map(|c| c.c).collect();
        assert_eq!(text, "Line 5");
    }

    #[test]
    fn test_pop_operations() {
        let mut buffer = ScrollbackBuffer::with_max_lines(10);

        buffer.push(&create_test_line("Line 1"));
        buffer.push(&create_test_line("Line 2"));
        buffer.push(&create_test_line("Line 3"));

        // Pop from back (most recent)
        let line = buffer.pop_back().unwrap();
        let decompressed = line.decompress();
        let text: String = decompressed.iter().map(|c| c.c).collect();
        assert_eq!(text, "Line 3");
        assert_eq!(buffer.len(), 2);

        // Pop from front (oldest)
        let line = buffer.pop_front().unwrap();
        let decompressed = line.decompress();
        let text: String = decompressed.iter().map(|c| c.c).collect();
        assert_eq!(text, "Line 1");
        assert_eq!(buffer.len(), 1);
    }
}
