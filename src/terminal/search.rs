//! Terminal text search functionality
//!
//! Provides regex-based search through terminal buffer content

use regex::Regex;

/// A single search match location
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    /// Line number (0-indexed)
    pub line: usize,
    /// Start column (0-indexed, byte offset)
    pub start_col: usize,
    /// End column (0-indexed, byte offset)
    pub end_col: usize,
}

/// Search state for terminal content
#[derive(Debug)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Whether to use regex mode
    pub regex_mode: bool,
    /// Whether search is case-sensitive
    pub case_sensitive: bool,
    /// All matches found
    pub matches: Vec<SearchMatch>,
    /// Currently selected match index
    pub current_match: Option<usize>,
    /// Compiled regex pattern
    compiled_regex: Option<Regex>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    /// Create a new search state
    pub fn new() -> Self {
        Self {
            query: String::new(),
            regex_mode: false,
            case_sensitive: false,
            matches: Vec::new(),
            current_match: None,
            compiled_regex: None,
        }
    }

    /// Set the search query and compile pattern
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.compile_pattern();
        self.matches.clear();
        self.current_match = None;
    }

    /// Toggle regex mode
    pub fn toggle_regex(&mut self) {
        self.regex_mode = !self.regex_mode;
        self.compile_pattern();
    }

    /// Toggle case sensitivity
    pub fn toggle_case_sensitive(&mut self) {
        self.case_sensitive = !self.case_sensitive;
        self.compile_pattern();
    }

    /// Compile the search pattern
    fn compile_pattern(&mut self) {
        if self.query.is_empty() {
            self.compiled_regex = None;
            return;
        }

        let pattern = if self.regex_mode {
            self.query.clone()
        } else {
            regex::escape(&self.query)
        };

        let pattern = if self.case_sensitive {
            pattern
        } else {
            format!("(?i){pattern}")
        };

        self.compiled_regex = Regex::new(&pattern).ok();
    }

    /// Check if we have a valid pattern
    pub fn has_pattern(&self) -> bool {
        self.compiled_regex.is_some()
    }

    /// Find matches in a single line
    pub fn find_in_line(&self, line_num: usize, text: &str) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        if let Some(regex) = &self.compiled_regex {
            for m in regex.find_iter(text) {
                matches.push(SearchMatch {
                    line: line_num,
                    start_col: m.start(),
                    end_col: m.end(),
                });
            }
        }
        matches
    }

    /// Add matches from a line to the internal matches list
    pub fn search_line(&mut self, line_num: usize, text: &str) {
        let line_matches = self.find_in_line(line_num, text);
        self.matches.extend(line_matches);
    }

    /// Navigate to next match
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_match = Some(match self.current_match {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        });
        self.current_match.and_then(|i| self.matches.get(i))
    }

    /// Navigate to previous match
    pub fn prev_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_match = Some(match self.current_match {
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
            None => self.matches.len() - 1,
        });
        self.current_match.and_then(|i| self.matches.get(i))
    }

    /// Get current match
    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_match.and_then(|i| self.matches.get(i))
    }

    /// Get match count
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get current match index (1-indexed for display)
    pub fn current_index(&self) -> Option<usize> {
        self.current_match.map(|i| i + 1)
    }

    /// Clear all matches
    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
        self.compiled_regex = None;
    }

    /// Check if a position is within any match
    pub fn is_match_at(&self, line: usize, col: usize) -> bool {
        self.matches
            .iter()
            .any(|m| m.line == line && col >= m.start_col && col < m.end_col)
    }

    /// Check if a position is within the current match
    pub fn is_current_match_at(&self, line: usize, col: usize) -> bool {
        self.current().is_some_and(|m| {
            m.line == line && col >= m.start_col && col < m.end_col
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let mut state = SearchState::new();
        state.set_query("test".to_string());

        let matches = state.find_in_line(0, "this is a test string");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start_col, 10);
        assert_eq!(matches[0].end_col, 14);
    }

    #[test]
    fn test_search_multiple_matches() {
        let mut state = SearchState::new();
        state.set_query("a".to_string());

        let matches = state.find_in_line(0, "abracadabra");
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut state = SearchState::new();
        state.set_query("TEST".to_string());
        // Default is case-insensitive

        let matches = state.find_in_line(0, "this is a test");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_search_case_sensitive() {
        let mut state = SearchState::new();
        state.case_sensitive = true;
        state.set_query("TEST".to_string());

        let matches = state.find_in_line(0, "this is a test");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_search_regex_mode() {
        let mut state = SearchState::new();
        state.regex_mode = true;
        state.set_query(r"\d+".to_string());

        let matches = state.find_in_line(0, "abc123def456");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_search_navigation() {
        let mut state = SearchState::new();
        state.set_query("x".to_string());
        state.search_line(0, "x x x");

        assert_eq!(state.match_count(), 3);

        state.next_match();
        assert_eq!(state.current_index(), Some(1));

        state.next_match();
        assert_eq!(state.current_index(), Some(2));

        state.next_match();
        assert_eq!(state.current_index(), Some(3));

        state.next_match(); // Wrap around
        assert_eq!(state.current_index(), Some(1));

        state.prev_match(); // Wrap back
        assert_eq!(state.current_index(), Some(3));
    }

    #[test]
    fn test_search_empty_query() {
        let mut state = SearchState::new();
        state.set_query("".to_string());

        assert!(!state.has_pattern());
        let matches = state.find_in_line(0, "any text");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_search_special_chars() {
        let mut state = SearchState::new();
        // Without regex mode, special chars are escaped
        state.set_query("[test]".to_string());

        let matches = state.find_in_line(0, "this is [test] text");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_is_match_at() {
        let mut state = SearchState::new();
        state.set_query("hello".to_string());
        state.search_line(0, "hello world");

        assert!(state.is_match_at(0, 0));
        assert!(state.is_match_at(0, 4));
        assert!(!state.is_match_at(0, 5));
        assert!(!state.is_match_at(1, 0));
    }
}
