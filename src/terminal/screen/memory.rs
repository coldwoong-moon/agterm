//! Memory optimization utilities for terminal screen buffers

use std::collections::HashMap;
use std::sync::Arc;

/// String interner for reducing memory usage of repeated strings (URLs, etc.)
#[derive(Debug, Clone)]
pub struct StringInterner {
    strings: HashMap<String, Arc<String>>,
    hits: usize,
    misses: usize,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Intern a string, returning an Arc to a shared copy
    pub fn intern(&mut self, s: String) -> Arc<String> {
        if let Some(existing) = self.strings.get(&s) {
            self.hits += 1;
            Arc::clone(existing)
        } else {
            self.misses += 1;
            let arc = Arc::new(s.clone());
            self.strings.insert(s, Arc::clone(&arc));
            arc
        }
    }

    /// Get statistics about string interning efficiency
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.strings.len(), self.hits, self.misses)
    }

    /// Clear old strings to prevent unbounded growth
    pub fn cleanup(&mut self) {
        // Remove strings that are only referenced by the interner
        self.strings.retain(|_, arc| Arc::strong_count(arc) > 1);
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.strings
            .iter()
            .map(|(k, v)| k.len() + v.len() + std::mem::size_of::<Arc<String>>())
            .sum()
    }
}

/// Compressed line representation using run-length encoding for empty cells
///
/// This enum provides memory-efficient storage for terminal lines by compressing
/// lines that contain mostly default (empty) cells. Reserved for future use.
#[derive(Clone, Debug)]
#[allow(dead_code)] // Reserved for future memory optimization features
pub enum CompressedLine {
    /// Uncompressed line (for lines with non-default cells)
    Raw(Vec<super::Cell>),
    /// Empty line (all default cells)
    Empty(usize), // number of cells
    /// Run-length encoded (not yet implemented - reserved for future optimization)
    #[allow(dead_code)]
    Rle {
        cells: Vec<super::Cell>,
        runs: Vec<(usize, usize)>, // (start_index, count)
    },
}

impl CompressedLine {
    /// Create a compressed line from a raw line
    #[allow(dead_code)] // Reserved for future use
    pub fn from_line(line: Vec<super::Cell>) -> Self {
        // Check if all cells are default
        if line.iter().all(|cell| is_default_cell(cell)) {
            CompressedLine::Empty(line.len())
        } else {
            CompressedLine::Raw(line)
        }
    }

    /// Decompress the line back to a raw line
    #[allow(dead_code)] // Reserved for future use
    pub fn to_line(&self) -> Vec<super::Cell> {
        match self {
            CompressedLine::Raw(line) => line.clone(),
            CompressedLine::Empty(len) => {
                vec![super::Cell::default(); *len]
            }
            CompressedLine::Rle { .. } => {
                // Not yet implemented
                vec![]
            }
        }
    }

    /// Get the approximate memory usage of this compressed line
    #[allow(dead_code)] // Reserved for future use
    pub fn memory_usage(&self) -> usize {
        match self {
            CompressedLine::Raw(line) => line.len() * std::mem::size_of::<super::Cell>(),
            CompressedLine::Empty(_) => std::mem::size_of::<usize>(),
            CompressedLine::Rle { cells, runs } => {
                cells.len() * std::mem::size_of::<super::Cell>()
                    + runs.len() * std::mem::size_of::<(usize, usize)>()
            }
        }
    }
}

/// Check if a cell is in default state (empty)
#[allow(dead_code)] // Used by CompressedLine (reserved for future)
fn is_default_cell(cell: &super::Cell) -> bool {
    cell.c == ' '
        && cell.fg.is_none()
        && cell.bg.is_none()
        && !cell.bold
        && !cell.underline
        && !cell.reverse
        && !cell.dim
        && !cell.italic
        && !cell.strikethrough
        && !cell.wide
        && !cell.placeholder
        && cell.hyperlink.is_none()
}

/// Memory usage statistics for the terminal screen
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Number of lines in main buffer
    pub buffer_lines: usize,
    /// Number of lines in scrollback
    pub scrollback_lines: usize,
    /// Approximate memory usage of main buffer (bytes)
    pub buffer_bytes: usize,
    /// Approximate memory usage of scrollback (bytes)
    pub scrollback_bytes: usize,
    /// Memory usage of string interner (bytes)
    pub interner_bytes: usize,
    /// Number of interned strings
    pub interned_strings: usize,
    /// String interner hit rate
    pub interner_hits: usize,
    /// String interner miss rate
    pub interner_misses: usize,
    /// Total memory usage (bytes)
    pub total_bytes: usize,
}

impl MemoryStats {
    /// Get a human-readable string representation
    pub fn to_string(&self) -> String {
        let buffer_kb = self.buffer_bytes / 1024;
        let scrollback_kb = self.scrollback_bytes / 1024;
        let interner_kb = self.interner_bytes / 1024;
        let total_kb = self.total_bytes / 1024;

        let hit_rate = if self.interner_hits + self.interner_misses > 0 {
            (self.interner_hits as f64) / ((self.interner_hits + self.interner_misses) as f64)
                * 100.0
        } else {
            0.0
        };

        format!(
            "Memory: {}KB total | Buffer: {}KB ({} lines) | Scrollback: {}KB ({} lines) | \
             Interner: {}KB ({} strings, {:.1}% hit rate)",
            total_kb,
            buffer_kb,
            self.buffer_lines,
            scrollback_kb,
            self.scrollback_lines,
            interner_kb,
            self.interned_strings,
            hit_rate
        )
    }
}

/// Calculate memory usage of a cell
pub fn cell_memory_size(cell: &super::Cell) -> usize {
    let base_size = std::mem::size_of::<super::Cell>();
    let hyperlink_size = cell.hyperlink.as_ref().map(|s| s.len()).unwrap_or(0);
    base_size + hyperlink_size
}

/// Calculate memory usage of a line
pub fn line_memory_size(line: &[super::Cell]) -> usize {
    line.iter().map(cell_memory_size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::screen::Cell;

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let url1 = "https://example.com".to_string();
        let url2 = "https://example.com".to_string();
        let url3 = "https://different.com".to_string();

        let arc1 = interner.intern(url1);
        let arc2 = interner.intern(url2);
        let arc3 = interner.intern(url3);

        // Same URLs should share the same Arc
        assert!(Arc::ptr_eq(&arc1, &arc2));
        assert!(!Arc::ptr_eq(&arc1, &arc3));

        let (strings, hits, misses) = interner.stats();
        assert_eq!(strings, 2);
        assert_eq!(hits, 1);
        assert_eq!(misses, 2);
    }

    #[test]
    fn test_compressed_line_empty() {
        let empty_line = vec![Cell::default(); 80];
        let compressed = CompressedLine::from_line(empty_line);

        match compressed {
            CompressedLine::Empty(len) => assert_eq!(len, 80),
            _ => panic!("Expected Empty variant"),
        }

        let decompressed = compressed.to_line();
        assert_eq!(decompressed.len(), 80);
    }

    #[test]
    fn test_memory_stats_display() {
        let stats = MemoryStats {
            buffer_lines: 24,
            scrollback_lines: 1000,
            buffer_bytes: 10240,
            scrollback_bytes: 102400,
            interner_bytes: 1024,
            interned_strings: 10,
            interner_hits: 100,
            interner_misses: 10,
            total_bytes: 113664,
        };

        let output = stats.to_string();
        assert!(output.contains("111KB total"));
        assert!(output.contains("24 lines"));
        assert!(output.contains("90.9% hit rate"));
    }
}
