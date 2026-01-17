//! Memory Block
//!
//! Structured units for context window management.
//! Based on Letta Memory Blocks pattern.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Memory block label - identifies the purpose of the block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryBlockLabel {
    /// Current working directory and environment info
    WorkingContext,
    /// Active task list (scratchpad)
    TaskProgress,
    /// Recent terminal output (sliding window)
    RecentOutput,
    /// Context from previous sessions
    SessionHistory,
    /// MCP server state and available tools
    McpContext,
}

impl std::fmt::Display for MemoryBlockLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkingContext => write!(f, "working_context"),
            Self::TaskProgress => write!(f, "task_progress"),
            Self::RecentOutput => write!(f, "recent_output"),
            Self::SessionHistory => write!(f, "session_history"),
            Self::McpContext => write!(f, "mcp_context"),
        }
    }
}

/// Memory Block - a structured unit in the context window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    /// Block label identifying its purpose
    pub label: MemoryBlockLabel,
    /// String representation of block data
    pub value: String,
    /// Maximum token limit
    pub token_limit: usize,
    /// Current token count (estimated)
    pub current_tokens: usize,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

impl MemoryBlock {
    /// Create a new memory block
    #[must_use]
    pub fn new(label: MemoryBlockLabel, token_limit: usize) -> Self {
        Self {
            label,
            value: String::new(),
            token_limit,
            current_tokens: 0,
            updated_at: Utc::now(),
        }
    }

    /// Update the block value
    pub fn update(&mut self, value: String) {
        self.current_tokens = estimate_tokens(&value);
        self.value = value;
        self.updated_at = Utc::now();
    }

    /// Append to the block value
    pub fn append(&mut self, text: &str) {
        self.value.push_str(text);
        self.current_tokens = estimate_tokens(&self.value);
        self.updated_at = Utc::now();
    }

    /// Check if the block is over the token limit
    #[must_use]
    pub fn is_over_limit(&self) -> bool {
        self.current_tokens > self.token_limit
    }

    /// Get remaining token capacity
    #[must_use]
    pub fn remaining_tokens(&self) -> usize {
        self.token_limit.saturating_sub(self.current_tokens)
    }

    /// Get usage ratio (0.0 - 1.0+)
    #[must_use]
    pub fn usage_ratio(&self) -> f64 {
        if self.token_limit == 0 {
            0.0
        } else {
            self.current_tokens as f64 / self.token_limit as f64
        }
    }

    /// Clear the block
    pub fn clear(&mut self) {
        self.value.clear();
        self.current_tokens = 0;
        self.updated_at = Utc::now();
    }

    /// Truncate to fit within token limit
    pub fn truncate_to_limit(&mut self) {
        if !self.is_over_limit() {
            return;
        }

        // Simple truncation: keep last N characters
        // This is approximate; a proper implementation would use tokenizer
        let chars_per_token = 4.0; // Rough estimate
        let target_chars = (self.token_limit as f64 * chars_per_token) as usize;

        if self.value.len() > target_chars {
            let truncated = format!(
                "... [truncated {} chars]\n{}",
                self.value.len() - target_chars,
                &self.value[self.value.len() - target_chars..]
            );
            self.value = truncated;
            self.current_tokens = estimate_tokens(&self.value);
            self.updated_at = Utc::now();
        }
    }
}

/// Estimate token count from string
/// Uses a simple heuristic: ~4 characters per token for English text
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    // More accurate heuristic:
    // - Words + punctuation + special characters
    // This is still approximate; use tiktoken for accuracy
    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();

    // Average of character-based and word-based estimates
    let char_estimate = char_count / 4;
    let word_estimate = (word_count as f64 * 1.3) as usize;

    (char_estimate + word_estimate) / 2
}

/// Default token limits for each block type
#[must_use]
pub fn default_token_limit(label: MemoryBlockLabel) -> usize {
    match label {
        MemoryBlockLabel::WorkingContext => 500,
        MemoryBlockLabel::TaskProgress => 2000,
        MemoryBlockLabel::RecentOutput => 4000,
        MemoryBlockLabel::SessionHistory => 2000,
        MemoryBlockLabel::McpContext => 1000,
    }
}

/// Memory block collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryBlocks {
    blocks: std::collections::HashMap<MemoryBlockLabel, MemoryBlock>,
}

impl MemoryBlocks {
    /// Create a new empty collection
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default blocks
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut blocks = Self::new();

        blocks.insert(MemoryBlock::new(
            MemoryBlockLabel::WorkingContext,
            default_token_limit(MemoryBlockLabel::WorkingContext),
        ));
        blocks.insert(MemoryBlock::new(
            MemoryBlockLabel::TaskProgress,
            default_token_limit(MemoryBlockLabel::TaskProgress),
        ));
        blocks.insert(MemoryBlock::new(
            MemoryBlockLabel::RecentOutput,
            default_token_limit(MemoryBlockLabel::RecentOutput),
        ));
        blocks.insert(MemoryBlock::new(
            MemoryBlockLabel::SessionHistory,
            default_token_limit(MemoryBlockLabel::SessionHistory),
        ));
        blocks.insert(MemoryBlock::new(
            MemoryBlockLabel::McpContext,
            default_token_limit(MemoryBlockLabel::McpContext),
        ));

        blocks
    }

    /// Insert a block
    pub fn insert(&mut self, block: MemoryBlock) {
        self.blocks.insert(block.label, block);
    }

    /// Get a block by label
    #[must_use]
    pub fn get(&self, label: MemoryBlockLabel) -> Option<&MemoryBlock> {
        self.blocks.get(&label)
    }

    /// Get a mutable block by label
    pub fn get_mut(&mut self, label: MemoryBlockLabel) -> Option<&mut MemoryBlock> {
        self.blocks.get_mut(&label)
    }

    /// Update a block's value
    pub fn update(&mut self, label: MemoryBlockLabel, value: String) {
        if let Some(block) = self.blocks.get_mut(&label) {
            block.update(value);
        }
    }

    /// Get total token count across all blocks
    #[must_use]
    pub fn total_tokens(&self) -> usize {
        self.blocks.values().map(|b| b.current_tokens).sum()
    }

    /// Get all blocks
    pub fn all(&self) -> impl Iterator<Item = &MemoryBlock> {
        self.blocks.values()
    }

    /// Render all blocks as formatted string
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::new();

        for block in self.blocks.values() {
            if !block.value.is_empty() {
                output.push_str(&format!("## {}\n{}\n\n", block.label, block.value));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_block_creation() {
        let block = MemoryBlock::new(MemoryBlockLabel::TaskProgress, 1000);

        assert_eq!(block.label, MemoryBlockLabel::TaskProgress);
        assert_eq!(block.token_limit, 1000);
        assert_eq!(block.current_tokens, 0);
        assert!(block.value.is_empty());
    }

    #[test]
    fn test_memory_block_update() {
        let mut block = MemoryBlock::new(MemoryBlockLabel::RecentOutput, 1000);
        block.update("Hello, world!".to_string());

        assert!(!block.value.is_empty());
        assert!(block.current_tokens > 0);
    }

    #[test]
    fn test_memory_block_append() {
        let mut block = MemoryBlock::new(MemoryBlockLabel::RecentOutput, 1000);
        block.append("Line 1\n");
        block.append("Line 2\n");

        assert!(block.value.contains("Line 1"));
        assert!(block.value.contains("Line 2"));
    }

    #[test]
    fn test_token_estimation() {
        let text = "Hello, world! This is a test.";
        let tokens = estimate_tokens(text);

        // Should be roughly 7-10 tokens
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_memory_blocks_collection() {
        let blocks = MemoryBlocks::with_defaults();

        assert!(blocks.get(MemoryBlockLabel::WorkingContext).is_some());
        assert!(blocks.get(MemoryBlockLabel::TaskProgress).is_some());
        assert!(blocks.get(MemoryBlockLabel::RecentOutput).is_some());
    }
}
