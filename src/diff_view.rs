//! Terminal Diff Viewer
//!
//! This module provides a comprehensive diff viewer for terminal output with:
//! - Myers diff algorithm implementation
//! - Side-by-side and unified view modes
//! - Syntax highlighting for changes
//! - Navigation between changes

use std::cmp::min;
use std::collections::HashMap;

/// Type of change in a diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Line was added in the new version
    Added,
    /// Line was removed from the old version
    Removed,
    /// Line exists in both versions unchanged
    Unchanged,
    /// Line was modified between versions
    Modified,
}

/// Represents a single line in a diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// Type of change
    pub line_type: DiffLineType,
    /// Content from the left/old version
    pub left_content: Option<String>,
    /// Content from the right/new version
    pub right_content: Option<String>,
    /// Line number in the left/old version (1-indexed, None if added)
    pub left_line_num: Option<usize>,
    /// Line number in the right/new version (1-indexed, None if removed)
    pub right_line_num: Option<usize>,
}

impl DiffLine {
    /// Create a new unchanged line
    pub fn unchanged(content: String, left_num: usize, right_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Unchanged,
            left_content: Some(content.clone()),
            right_content: Some(content),
            left_line_num: Some(left_num),
            right_line_num: Some(right_num),
        }
    }

    /// Create a new added line
    pub fn added(content: String, right_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Added,
            left_content: None,
            right_content: Some(content),
            left_line_num: None,
            right_line_num: Some(right_num),
        }
    }

    /// Create a new removed line
    pub fn removed(content: String, left_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Removed,
            left_content: Some(content),
            right_content: None,
            left_line_num: Some(left_num),
            right_line_num: None,
        }
    }

    /// Create a new modified line
    pub fn modified(left_content: String, right_content: String, left_num: usize, right_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Modified,
            left_content: Some(left_content),
            right_content: Some(right_content),
            left_line_num: Some(left_num),
            right_line_num: Some(right_num),
        }
    }
}

/// Statistics about changes in a diff
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiffStats {
    /// Number of lines added
    pub added: usize,
    /// Number of lines removed
    pub removed: usize,
    /// Number of lines modified
    pub modified: usize,
    /// Number of lines unchanged
    pub unchanged: usize,
}

impl DiffStats {
    /// Get total number of changes (added + removed + modified)
    pub fn total_changes(&self) -> usize {
        self.added + self.removed + self.modified
    }

    /// Get total number of lines
    pub fn total_lines(&self) -> usize {
        self.added + self.removed + self.modified + self.unchanged
    }
}

/// Result of a diff operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    /// List of diff lines
    pub lines: Vec<DiffLine>,
    /// Statistics about the diff
    pub stats: DiffStats,
}

impl DiffResult {
    /// Create a new diff result from lines
    pub fn new(lines: Vec<DiffLine>) -> Self {
        let mut stats = DiffStats::default();

        for line in &lines {
            match line.line_type {
                DiffLineType::Added => stats.added += 1,
                DiffLineType::Removed => stats.removed += 1,
                DiffLineType::Modified => stats.modified += 1,
                DiffLineType::Unchanged => stats.unchanged += 1,
            }
        }

        Self { lines, stats }
    }

    /// Get indices of all change lines (non-unchanged)
    pub fn change_indices(&self) -> Vec<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.line_type != DiffLineType::Unchanged)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Find the next change after the given index
    pub fn next_change(&self, current_idx: usize) -> Option<usize> {
        self.change_indices()
            .into_iter()
            .find(|&idx| idx > current_idx)
    }

    /// Find the previous change before the given index
    pub fn prev_change(&self, current_idx: usize) -> Option<usize> {
        self.change_indices()
            .into_iter()
            .rev()
            .find(|&idx| idx < current_idx)
    }
}

/// Myers diff algorithm implementation
///
/// This implements the O(ND) diff algorithm by Eugene W. Myers.
/// Reference: "An O(ND) Difference Algorithm and Its Variations"
pub struct MyersDiff {
    old: Vec<String>,
    new: Vec<String>,
}

/// Represents an edit operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edit {
    Insert(usize), // Insert from new at position y
    Delete(usize), // Delete from old at position x
    Keep(usize, usize), // Keep both at positions (x, y)
}

impl MyersDiff {
    /// Create a new Myers diff instance
    pub fn new(old: Vec<String>, new: Vec<String>) -> Self {
        Self { old, new }
    }

    /// Compute the diff using Myers algorithm
    pub fn compute(&self) -> DiffResult {
        let edits = self.shortest_edit_script();
        self.edits_to_diff_result(edits)
    }

    /// Find the shortest edit script using Myers algorithm
    fn shortest_edit_script(&self) -> Vec<Edit> {
        let n = self.old.len();
        let m = self.new.len();
        let max = n + m;

        // V[k] = x where k = x - y
        let mut v: HashMap<isize, usize> = HashMap::new();
        v.insert(1, 0);

        // Trace for backtracking
        let mut trace: Vec<HashMap<isize, usize>> = Vec::new();

        for d in 0..=max {
            let mut current_v = v.clone();

            let k_start = -(d as isize);
            let k_end = d as isize;

            for k in (k_start..=k_end).step_by(2) {
                // Choose whether to move down or right
                let mut x = if k == k_start || (k != k_end && v.get(&(k - 1)).unwrap_or(&0) < v.get(&(k + 1)).unwrap_or(&0)) {
                    // Move down (insert)
                    *v.get(&(k + 1)).unwrap_or(&0)
                } else {
                    // Move right (delete)
                    v.get(&(k - 1)).unwrap_or(&0) + 1
                };

                let mut y = (x as isize - k) as usize;

                // Follow diagonal (matching lines)
                while x < n && y < m && self.old[x] == self.new[y] {
                    x += 1;
                    y += 1;
                }

                current_v.insert(k, x);

                // Check if we reached the end
                if x >= n && y >= m {
                    trace.push(current_v);
                    return self.backtrack(trace);
                }
            }

            trace.push(current_v.clone());
            v = current_v;
        }

        // If we reach here, return empty edits (shouldn't happen)
        Vec::new()
    }

    /// Backtrack through the trace to build the edit script
    fn backtrack(&self, trace: Vec<HashMap<isize, usize>>) -> Vec<Edit> {
        let mut edits = Vec::new();
        let mut x = self.old.len();
        let mut y = self.new.len();

        for d in (0..trace.len()).rev() {
            let v = &trace[d];
            let k = x as isize - y as isize;

            // At d=0, there is no previous step, so we need to handle it specially
            if d == 0 {
                // All remaining lines are kept (diagonal match from origin)
                while x > 0 && y > 0 {
                    x -= 1;
                    y -= 1;
                    edits.push(Edit::Keep(x, y));
                }
                break;
            }

            let prev_k = if k == -(d as isize) || (k != d as isize && v.get(&(k - 1)).unwrap_or(&0) < v.get(&(k + 1)).unwrap_or(&0)) {
                k + 1
            } else {
                k - 1
            };

            let prev_x = *v.get(&prev_k).unwrap_or(&0);
            let prev_y = (prev_x as isize - prev_k) as usize;

            // Follow diagonals backward
            while x > prev_x && y > prev_y {
                x -= 1;
                y -= 1;
                edits.push(Edit::Keep(x, y));
            }

            if x == prev_x {
                // Insert
                y -= 1;
                edits.push(Edit::Insert(y));
            } else {
                // Delete
                x -= 1;
                edits.push(Edit::Delete(x));
            }
        }

        edits.reverse();
        edits
    }

    /// Convert edit script to diff result with change detection
    fn edits_to_diff_result(&self, edits: Vec<Edit>) -> DiffResult {
        let mut lines = Vec::new();
        let mut pending_delete: Option<(String, usize)> = None;

        let mut left_num = 1;
        let mut right_num = 1;

        for edit in edits {
            match edit {
                Edit::Keep(x, _y) => {
                    // If there was a pending delete, emit it as removed
                    if let Some((content, num)) = pending_delete.take() {
                        lines.push(DiffLine::removed(content, num));
                    }

                    lines.push(DiffLine::unchanged(
                        self.old[x].clone(),
                        left_num,
                        right_num,
                    ));
                    left_num += 1;
                    right_num += 1;
                }
                Edit::Delete(x) => {
                    // Store delete, might be part of a modification
                    if pending_delete.is_some() {
                        // Multiple deletes in a row
                        if let Some((content, num)) = pending_delete.take() {
                            lines.push(DiffLine::removed(content, num));
                        }
                    }
                    pending_delete = Some((self.old[x].clone(), left_num));
                    left_num += 1;
                }
                Edit::Insert(y) => {
                    // Check if this is a modification (delete + insert)
                    if let Some((left_content, left_num_val)) = pending_delete.take() {
                        lines.push(DiffLine::modified(
                            left_content,
                            self.new[y].clone(),
                            left_num_val,
                            right_num,
                        ));
                    } else {
                        lines.push(DiffLine::added(self.new[y].clone(), right_num));
                    }
                    right_num += 1;
                }
            }
        }

        // Handle any remaining pending delete
        if let Some((content, num)) = pending_delete {
            lines.push(DiffLine::removed(content, num));
        }

        DiffResult::new(lines)
    }
}

/// View mode for displaying diffs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    /// Side-by-side comparison
    SideBySide,
    /// Unified diff format
    Unified,
}

/// Diff viewer for terminal display
pub struct DiffViewer {
    /// The diff result to display
    result: DiffResult,
    /// Current view mode
    mode: DiffViewMode,
    /// Current line being viewed (for navigation)
    current_line: usize,
    /// Width of the terminal
    terminal_width: usize,
}

impl DiffViewer {
    /// Create a new diff viewer
    pub fn new(result: DiffResult, terminal_width: usize) -> Self {
        Self {
            result,
            mode: DiffViewMode::SideBySide,
            current_line: 0,
            terminal_width,
        }
    }

    /// Set the view mode
    pub fn set_mode(&mut self, mode: DiffViewMode) {
        self.mode = mode;
    }

    /// Navigate to the next change
    pub fn next_change(&mut self) -> bool {
        if let Some(idx) = self.result.next_change(self.current_line) {
            self.current_line = idx;
            true
        } else {
            false
        }
    }

    /// Navigate to the previous change
    pub fn prev_change(&mut self) -> bool {
        if let Some(idx) = self.result.prev_change(self.current_line) {
            self.current_line = idx;
            true
        } else {
            false
        }
    }

    /// Get the current line index
    pub fn current_line(&self) -> usize {
        self.current_line
    }

    /// Set the current line index
    pub fn set_current_line(&mut self, line: usize) {
        if line < self.result.lines.len() {
            self.current_line = line;
        }
    }

    /// Render the diff to a string with ANSI color codes
    pub fn render(&self) -> String {
        match self.mode {
            DiffViewMode::SideBySide => self.render_side_by_side(),
            DiffViewMode::Unified => self.render_unified(),
        }
    }

    /// Render side-by-side view
    fn render_side_by_side(&self) -> String {
        let mut output = String::new();
        let half_width = (self.terminal_width - 3) / 2; // -3 for separator " | "

        // Header
        output.push_str(&format!(
            "\x1b[1m{:^width$} | {:^width$}\x1b[0m\n",
            "Old",
            "New",
            width = half_width
        ));
        output.push_str(&format!("{}\n", "=".repeat(self.terminal_width)));

        // Lines
        for (idx, line) in self.result.lines.iter().enumerate() {
            let is_current = idx == self.current_line;
            let prefix = if is_current { ">" } else { " " };

            match line.line_type {
                DiffLineType::Unchanged => {
                    let content = line.left_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, half_width - 2);
                    output.push_str(&format!(
                        "{} {:4} {} | {:4} {}\n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated,
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated
                    ));
                }
                DiffLineType::Added => {
                    let content = line.right_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, half_width - 2);
                    output.push_str(&format!(
                        "{} {:4} {:width$} | \x1b[32m{:4} +{}\x1b[0m\n",
                        prefix,
                        "    ",
                        "",
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated,
                        width = half_width - 6
                    ));
                }
                DiffLineType::Removed => {
                    let content = line.left_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, half_width - 2);
                    output.push_str(&format!(
                        "{} \x1b[31m{:4} -{}\x1b[0m {:width$} | {:4} \n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated,
                        "",
                        "    ",
                        width = half_width - 6
                    ));
                }
                DiffLineType::Modified => {
                    let left = line.left_content.as_deref().unwrap_or("");
                    let right = line.right_content.as_deref().unwrap_or("");
                    let left_trunc = self.truncate(left, half_width - 2);
                    let right_trunc = self.truncate(right, half_width - 2);
                    output.push_str(&format!(
                        "{} \x1b[33m{:4} ~{}\x1b[0m | \x1b[33m{:4} ~{}\x1b[0m\n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        left_trunc,
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        right_trunc
                    ));
                }
            }
        }

        // Footer with stats
        output.push_str(&format!("{}\n", "=".repeat(self.terminal_width)));
        output.push_str(&format!(
            "\x1b[32m+{}\x1b[0m \x1b[31m-{}\x1b[0m \x1b[33m~{}\x1b[0m (Total: {} changes)\n",
            self.result.stats.added,
            self.result.stats.removed,
            self.result.stats.modified,
            self.result.stats.total_changes()
        ));

        output
    }

    /// Render unified diff view
    fn render_unified(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("\x1b[1mUnified Diff\x1b[0m\n");
        output.push_str(&format!("{}\n", "=".repeat(self.terminal_width)));

        // Lines
        for (idx, line) in self.result.lines.iter().enumerate() {
            let is_current = idx == self.current_line;
            let prefix = if is_current { ">" } else { " " };

            match line.line_type {
                DiffLineType::Unchanged => {
                    let content = line.left_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, self.terminal_width - 10);
                    output.push_str(&format!(
                        "{} {:4} {:4}   {}\n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated
                    ));
                }
                DiffLineType::Added => {
                    let content = line.right_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, self.terminal_width - 10);
                    output.push_str(&format!(
                        "{} {:4} \x1b[32m{:4} + {}\x1b[0m\n",
                        prefix,
                        "    ",
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        truncated
                    ));
                }
                DiffLineType::Removed => {
                    let content = line.left_content.as_deref().unwrap_or("");
                    let truncated = self.truncate(content, self.terminal_width - 10);
                    output.push_str(&format!(
                        "{} \x1b[31m{:4}\x1b[0m {:4} \x1b[31m- {}\x1b[0m\n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        "    ",
                        truncated
                    ));
                }
                DiffLineType::Modified => {
                    let left = line.left_content.as_deref().unwrap_or("");
                    let right = line.right_content.as_deref().unwrap_or("");
                    let left_trunc = self.truncate(left, self.terminal_width - 10);
                    let right_trunc = self.truncate(right, self.terminal_width - 10);
                    output.push_str(&format!(
                        "{} \x1b[33m{:4}\x1b[0m {:4} \x1b[33m- {}\x1b[0m\n",
                        prefix,
                        line.left_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        "    ",
                        left_trunc
                    ));
                    output.push_str(&format!(
                        "{} {:4} \x1b[33m{:4} + {}\x1b[0m\n",
                        prefix,
                        "    ",
                        line.right_line_num.map_or("    ".to_string(), |n| format!("{n:4}")),
                        right_trunc
                    ));
                }
            }
        }

        // Footer with stats
        output.push_str(&format!("{}\n", "=".repeat(self.terminal_width)));
        output.push_str(&format!(
            "\x1b[32m+{}\x1b[0m \x1b[31m-{}\x1b[0m \x1b[33m~{}\x1b[0m (Total: {} changes)\n",
            self.result.stats.added,
            self.result.stats.removed,
            self.result.stats.modified,
            self.result.stats.total_changes()
        ));

        output
    }

    /// Truncate a string to fit within the given width
    fn truncate(&self, s: &str, width: usize) -> String {
        if s.len() <= width {
            format!("{s:width$}")
        } else {
            format!("{}...", &s[..min(width.saturating_sub(3), s.len())])
        }
    }

    /// Get reference to the diff result
    pub fn result(&self) -> &DiffResult {
        &self.result
    }
}

/// Convenience function to compute and display a diff
pub fn diff_strings(old: &str, new: &str, terminal_width: usize) -> String {
    let old_lines: Vec<String> = old.lines().map(|s| s.to_string()).collect();
    let new_lines: Vec<String> = new.lines().map(|s| s.to_string()).collect();

    let diff = MyersDiff::new(old_lines, new_lines);
    let result = diff.compute();

    let viewer = DiffViewer::new(result, terminal_width);
    viewer.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_line_creation() {
        let unchanged = DiffLine::unchanged("test".to_string(), 1, 1);
        assert_eq!(unchanged.line_type, DiffLineType::Unchanged);
        assert_eq!(unchanged.left_line_num, Some(1));
        assert_eq!(unchanged.right_line_num, Some(1));

        let added = DiffLine::added("new".to_string(), 2);
        assert_eq!(added.line_type, DiffLineType::Added);
        assert_eq!(added.left_line_num, None);
        assert_eq!(added.right_line_num, Some(2));

        let removed = DiffLine::removed("old".to_string(), 3);
        assert_eq!(removed.line_type, DiffLineType::Removed);
        assert_eq!(removed.left_line_num, Some(3));
        assert_eq!(removed.right_line_num, None);

        let modified = DiffLine::modified("old".to_string(), "new".to_string(), 4, 5);
        assert_eq!(modified.line_type, DiffLineType::Modified);
        assert_eq!(modified.left_line_num, Some(4));
        assert_eq!(modified.right_line_num, Some(5));
    }

    #[test]
    fn test_diff_stats() {
        let lines = vec![
            DiffLine::unchanged("line1".to_string(), 1, 1),
            DiffLine::added("line2".to_string(), 2),
            DiffLine::removed("line3".to_string(), 2),
            DiffLine::modified("old".to_string(), "new".to_string(), 3, 3),
        ];

        let result = DiffResult::new(lines);
        assert_eq!(result.stats.unchanged, 1);
        assert_eq!(result.stats.added, 1);
        assert_eq!(result.stats.removed, 1);
        assert_eq!(result.stats.modified, 1);
        assert_eq!(result.stats.total_changes(), 3);
        assert_eq!(result.stats.total_lines(), 4);
    }

    #[test]
    fn test_myers_diff_identical() {
        let old = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
        let new = old.clone();

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        assert_eq!(result.stats.added, 0);
        assert_eq!(result.stats.removed, 0);
        assert_eq!(result.stats.modified, 0);
        assert_eq!(result.stats.unchanged, 3);
    }

    #[test]
    fn test_myers_diff_all_added() {
        let old = vec![];
        let new = vec!["line1".to_string(), "line2".to_string()];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        assert_eq!(result.stats.added, 2);
        assert_eq!(result.stats.removed, 0);
    }

    #[test]
    fn test_myers_diff_all_removed() {
        let old = vec!["line1".to_string(), "line2".to_string()];
        let new = vec![];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        assert_eq!(result.stats.removed, 2);
        assert_eq!(result.stats.added, 0);
    }

    #[test]
    fn test_myers_diff_mixed_changes() {
        let old = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
            "line4".to_string(),
        ];
        let new = vec![
            "line1".to_string(),
            "line2_modified".to_string(),
            "line3".to_string(),
            "line5".to_string(),
        ];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        // line1: unchanged
        // line2 -> line2_modified: modified (delete + insert)
        // line3: unchanged
        // line4 -> line5: modified (delete + insert)

        assert_eq!(result.stats.unchanged, 2);
        assert_eq!(result.stats.modified, 2);
    }

    #[test]
    fn test_myers_diff_complex() {
        let old = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let new = vec![
            "a".to_string(),
            "x".to_string(),
            "c".to_string(),
            "y".to_string(),
        ];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        // a: unchanged
        // b -> x: modified
        // c: unchanged
        // d -> y: modified

        assert_eq!(result.stats.unchanged, 2);
        assert_eq!(result.stats.modified, 2);
    }

    #[test]
    fn test_diff_result_navigation() {
        let lines = vec![
            DiffLine::unchanged("line1".to_string(), 1, 1),
            DiffLine::added("line2".to_string(), 2),
            DiffLine::unchanged("line3".to_string(), 2, 3),
            DiffLine::removed("line4".to_string(), 3),
            DiffLine::unchanged("line5".to_string(), 4, 4),
        ];

        let result = DiffResult::new(lines);
        let changes = result.change_indices();
        assert_eq!(changes, vec![1, 3]);

        assert_eq!(result.next_change(0), Some(1));
        assert_eq!(result.next_change(1), Some(3));
        assert_eq!(result.next_change(3), None);

        assert_eq!(result.prev_change(4), Some(3));
        assert_eq!(result.prev_change(3), Some(1));
        assert_eq!(result.prev_change(1), None);
    }

    #[test]
    fn test_diff_viewer_navigation() {
        let lines = vec![
            DiffLine::unchanged("line1".to_string(), 1, 1),
            DiffLine::added("line2".to_string(), 2),
            DiffLine::unchanged("line3".to_string(), 2, 3),
            DiffLine::removed("line4".to_string(), 3),
        ];

        let result = DiffResult::new(lines);
        let mut viewer = DiffViewer::new(result, 80);

        assert_eq!(viewer.current_line(), 0);

        assert!(viewer.next_change());
        assert_eq!(viewer.current_line(), 1);

        assert!(viewer.next_change());
        assert_eq!(viewer.current_line(), 3);

        assert!(!viewer.next_change());
        assert_eq!(viewer.current_line(), 3);

        assert!(viewer.prev_change());
        assert_eq!(viewer.current_line(), 1);
    }

    #[test]
    fn test_diff_viewer_modes() {
        let lines = vec![
            DiffLine::unchanged("line1".to_string(), 1, 1),
            DiffLine::added("line2".to_string(), 2),
        ];

        let result = DiffResult::new(lines);
        let mut viewer = DiffViewer::new(result, 80);

        // Test side-by-side mode
        viewer.set_mode(DiffViewMode::SideBySide);
        let output = viewer.render();
        assert!(output.contains("Old"));
        assert!(output.contains("New"));
        assert!(output.contains("|"));

        // Test unified mode
        viewer.set_mode(DiffViewMode::Unified);
        let output = viewer.render();
        assert!(output.contains("Unified Diff"));
    }

    #[test]
    fn test_diff_strings_convenience() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2_modified\nline3";

        let output = diff_strings(old, new, 80);
        assert!(!output.is_empty());
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
    }

    #[test]
    fn test_empty_diff() {
        let old = vec![];
        let new = vec![];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        assert_eq!(result.stats.total_lines(), 0);
        assert_eq!(result.stats.total_changes(), 0);
    }

    #[test]
    fn test_single_line_diff() {
        let old = vec!["line1".to_string()];
        let new = vec!["line2".to_string()];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        assert_eq!(result.stats.modified, 1);
        assert_eq!(result.lines.len(), 1);
    }

    #[test]
    fn test_viewer_truncation() {
        let lines = vec![
            DiffLine::unchanged("a".repeat(100), 1, 1),
        ];

        let result = DiffResult::new(lines);
        let viewer = DiffViewer::new(result, 40);
        let output = viewer.render();

        // Verify that long lines are truncated
        assert!(output.lines().all(|line| {
            // Remove ANSI codes for length calculation
            let clean = line.replace("\x1b[0m", "")
                .replace("\x1b[1m", "")
                .replace("\x1b[31m", "")
                .replace("\x1b[32m", "")
                .replace("\x1b[33m", "");
            clean.len() <= 100 // Allow some margin for formatting
        }));
    }

    #[test]
    fn test_line_numbers() {
        let old = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let new = vec!["a".to_string(), "x".to_string(), "c".to_string(), "d".to_string()];

        let diff = MyersDiff::new(old, new);
        let result = diff.compute();

        // Check that line numbers are properly assigned
        for line in &result.lines {
            match line.line_type {
                DiffLineType::Unchanged => {
                    assert!(line.left_line_num.is_some());
                    assert!(line.right_line_num.is_some());
                }
                DiffLineType::Added => {
                    assert!(line.left_line_num.is_none());
                    assert!(line.right_line_num.is_some());
                }
                DiffLineType::Removed => {
                    assert!(line.left_line_num.is_some());
                    assert!(line.right_line_num.is_none());
                }
                DiffLineType::Modified => {
                    assert!(line.left_line_num.is_some());
                    assert!(line.right_line_num.is_some());
                }
            }
        }
    }
}
