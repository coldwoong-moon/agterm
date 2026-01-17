//! Memory and Context Management
//!
//! Memory blocks and compaction strategies for context engineering.

pub mod block;
pub mod compactor;

pub use block::{default_token_limit, estimate_tokens, MemoryBlock, MemoryBlockLabel, MemoryBlocks};
pub use compactor::{
    generate_summary_prompt, CompactedOutput, CompactionConfig, Compactor, SummaryRequest,
    SummaryResponse, SummaryType,
};
