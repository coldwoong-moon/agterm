//! Execution History Module for AgTerm
//!
//! Tracks command execution history with risk levels, timestamps, and execution results.
//! Provides filtering and statistics capabilities for monitoring agent activity.

use chrono::{DateTime, Utc};
use floem::reactive::{RwSignal, SignalUpdate, SignalWith};
use uuid::Uuid;

use super::async_bridge::RiskLevel;

/// Individual history entry for a completed command
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Unique identifier for this entry
    pub id: Uuid,
    /// Agent ID that executed the command
    pub agent_id: String,
    /// The command that was executed
    pub command: String,
    /// Risk level of the command
    pub risk_level: RiskLevel,
    /// When the command was executed
    pub executed_at: DateTime<Utc>,
    /// How long the command took to execute (in milliseconds)
    pub duration_ms: u64,
    /// Exit code if available (None if still running or no exit code)
    pub exit_code: Option<i32>,
    /// Preview of the output (first 100 characters)
    pub output_preview: String,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        agent_id: String,
        command: String,
        risk_level: RiskLevel,
        duration_ms: u64,
        exit_code: Option<i32>,
        output: String,
    ) -> Self {
        let output_preview = if output.len() > 100 {
            format!("{}...", &output[..100])
        } else {
            output
        };

        Self {
            id: Uuid::new_v4(),
            agent_id,
            command,
            risk_level,
            executed_at: Utc::now(),
            duration_ms,
            exit_code,
            output_preview,
        }
    }

    /// Check if the command was successful (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Check if the command failed (non-zero exit code)
    pub fn is_failure(&self) -> bool {
        matches!(self.exit_code, Some(code) if code != 0)
    }
}

/// Statistics about execution history
#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    /// Total number of commands executed
    pub total_commands: usize,
    /// Number of commands executed today
    pub today_commands: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Most frequently used agent (if any)
    pub most_used_agent: Option<String>,
}

impl HistoryStats {
    /// Create empty statistics
    pub fn empty() -> Self {
        Self {
            total_commands: 0,
            today_commands: 0,
            success_rate: 0.0,
            most_used_agent: None,
        }
    }
}

/// Execution history manager with reactive state
#[derive(Clone)]
pub struct ExecutionHistory {
    /// List of all history entries (newest first)
    pub entries: RwSignal<Vec<HistoryEntry>>,
    /// Maximum number of entries to keep
    pub max_entries: usize,
}

impl ExecutionHistory {
    /// Create a new execution history with default max entries (1000)
    pub fn new() -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
            max_entries: 1000,
        }
    }

    /// Create a new execution history with custom max entries
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
            max_entries,
        }
    }

    /// Add a new entry to the history
    ///
    /// If the history exceeds max_entries, the oldest entries will be removed.
    pub fn add(&self, entry: HistoryEntry) {
        self.entries.update(|entries| {
            // Insert at the beginning (newest first)
            entries.insert(0, entry);

            // Trim to max size if needed
            if entries.len() > self.max_entries {
                entries.truncate(self.max_entries);
            }
        });
    }

    /// Search for entries matching a query string (case-insensitive)
    ///
    /// Searches in command text and agent ID.
    pub fn search(&self, query: &str) -> Vec<HistoryEntry> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| {
                    entry.command.to_lowercase().contains(&query_lower)
                        || entry.agent_id.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect()
        })
    }

    /// Get all entries from a specific agent
    pub fn by_agent(&self, agent_id: &str) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.agent_id == agent_id)
                .cloned()
                .collect()
        })
    }

    /// Get entries within a date range (inclusive)
    pub fn by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.executed_at >= start && entry.executed_at <= end)
                .cloned()
                .collect()
        })
    }

    /// Get all entries executed today
    pub fn today(&self) -> Vec<HistoryEntry> {
        let now = Utc::now();
        let start_of_day = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or(now);

        self.by_date_range(start_of_day, now)
    }

    /// Get entries by risk level
    pub fn by_risk_level(&self, risk_level: RiskLevel) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.risk_level == risk_level)
                .cloned()
                .collect()
        })
    }

    /// Get only successful commands
    pub fn successful(&self) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.is_success())
                .cloned()
                .collect()
        })
    }

    /// Get only failed commands
    pub fn failed(&self) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.is_failure())
                .cloned()
                .collect()
        })
    }

    /// Get statistics about the execution history
    pub fn get_stats(&self) -> HistoryStats {
        self.entries.with(|entries| {
            if entries.is_empty() {
                return HistoryStats::empty();
            }

            let total_commands = entries.len();
            let today_commands = self.today().len();

            // Calculate success rate (only count entries with exit codes)
            let (successful, total_with_exit_code) = entries.iter().fold((0, 0), |(succ, total), entry| {
                if let Some(exit_code) = entry.exit_code {
                    (
                        succ + if exit_code == 0 { 1 } else { 0 },
                        total + 1,
                    )
                } else {
                    (succ, total)
                }
            });

            let success_rate = if total_with_exit_code > 0 {
                successful as f64 / total_with_exit_code as f64
            } else {
                0.0
            };

            // Find most used agent
            let mut agent_counts = std::collections::HashMap::new();
            for entry in entries.iter() {
                *agent_counts.entry(entry.agent_id.clone()).or_insert(0) += 1;
            }

            let most_used_agent = agent_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(agent, _)| agent);

            HistoryStats {
                total_commands,
                today_commands,
                success_rate,
                most_used_agent,
            }
        })
    }

    /// Clear all history entries
    pub fn clear(&self) {
        self.entries.update(|entries| {
            entries.clear();
        });
    }

    /// Get the N most recent entries
    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        self.entries.with(|entries| {
            entries.iter().take(limit).cloned().collect()
        })
    }

    /// Get total count of entries
    pub fn count(&self) -> usize {
        self.entries.with(|entries| entries.len())
    }
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

// Integration with ExecutionPipeline
use super::execution_pipeline::{PipelineItem, PipelineStage};
use floem::reactive::SignalGet;

impl From<&PipelineItem> for HistoryEntry {
    /// Convert a completed PipelineItem to a HistoryEntry
    ///
    /// Only PipelineItems in Completed or Failed state should be converted to history entries.
    /// This implementation extracts the necessary information and calculates the duration.
    fn from(item: &PipelineItem) -> Self {
        let stage = item.stage.get();
        let exit_code = item.exit_code.get();
        let output = item.output.get().unwrap_or_default();

        // Calculate duration from created_at to now
        let duration = Utc::now()
            .signed_duration_since(item.created_at)
            .num_milliseconds()
            .max(0) as u64;

        // Determine exit code based on stage if not explicitly set
        let final_exit_code = exit_code.or_else(|| {
            match stage {
                PipelineStage::Completed => Some(0),
                PipelineStage::Failed => Some(1),
                PipelineStage::Cancelled => Some(130), // SIGINT exit code
                _ => None,
            }
        });

        Self::new(
            item.agent_id.clone(),
            item.command.clone(),
            item.risk_level,
            duration,
            final_exit_code,
            output,
        )
    }
}

impl From<PipelineItem> for HistoryEntry {
    /// Convert an owned PipelineItem to a HistoryEntry
    fn from(item: PipelineItem) -> Self {
        Self::from(&item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(
        agent_id: &str,
        command: &str,
        risk_level: RiskLevel,
        exit_code: Option<i32>,
    ) -> HistoryEntry {
        HistoryEntry::new(
            agent_id.to_string(),
            command.to_string(),
            risk_level,
            100,
            exit_code,
            "test output".to_string(),
        )
    }

    #[test]
    fn test_new_history() {
        let history = ExecutionHistory::new();
        assert_eq!(history.count(), 0);
        assert_eq!(history.max_entries, 1000);
    }

    #[test]
    fn test_add_entry() {
        let history = ExecutionHistory::new();
        let entry = create_test_entry("agent1", "ls -la", RiskLevel::Low, Some(0));

        history.add(entry);
        assert_eq!(history.count(), 1);
    }

    #[test]
    fn test_max_entries_limit() {
        let history = ExecutionHistory::with_capacity(3);

        for i in 0..5 {
            let entry = create_test_entry("agent1", &format!("command{}", i), RiskLevel::Low, Some(0));
            history.add(entry);
        }

        assert_eq!(history.count(), 3);
        // Most recent should be command4, command3, command2
        let entries = history.recent(3);
        assert_eq!(entries[0].command, "command4");
        assert_eq!(entries[1].command, "command3");
        assert_eq!(entries[2].command, "command2");
    }

    #[test]
    fn test_search() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls -la", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "rm file.txt", RiskLevel::High, Some(0)));
        history.add(create_test_entry("agent1", "cat README.md", RiskLevel::Low, Some(0)));

        let results = history.search("ls");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "ls -la");

        let results = history.search("agent1");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_by_agent() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "pwd", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent1", "cat file", RiskLevel::Low, Some(0)));

        let agent1_entries = history.by_agent("agent1");
        assert_eq!(agent1_entries.len(), 2);

        let agent2_entries = history.by_agent("agent2");
        assert_eq!(agent2_entries.len(), 1);
    }

    #[test]
    fn test_by_risk_level() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "rm file", RiskLevel::High, Some(0)));
        history.add(create_test_entry("agent3", "rm -rf /", RiskLevel::Critical, Some(1)));

        let low_risk = history.by_risk_level(RiskLevel::Low);
        assert_eq!(low_risk.len(), 1);

        let high_risk = history.by_risk_level(RiskLevel::High);
        assert_eq!(high_risk.len(), 1);

        let critical_risk = history.by_risk_level(RiskLevel::Critical);
        assert_eq!(critical_risk.len(), 1);
    }

    #[test]
    fn test_successful_and_failed() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "failed_cmd", RiskLevel::Low, Some(1)));
        history.add(create_test_entry("agent3", "another_success", RiskLevel::Low, Some(0)));

        let successful = history.successful();
        assert_eq!(successful.len(), 2);

        let failed = history.failed();
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn test_get_stats() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent1", "pwd", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "failed", RiskLevel::High, Some(1)));

        let stats = history.get_stats();
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.success_rate, 2.0 / 3.0);
        assert_eq!(stats.most_used_agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_clear() {
        let history = ExecutionHistory::new();
        history.add(create_test_entry("agent1", "ls", RiskLevel::Low, Some(0)));
        history.add(create_test_entry("agent2", "pwd", RiskLevel::Low, Some(0)));

        assert_eq!(history.count(), 2);

        history.clear();
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_recent() {
        let history = ExecutionHistory::new();
        for i in 0..10 {
            history.add(create_test_entry("agent1", &format!("cmd{}", i), RiskLevel::Low, Some(0)));
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].command, "cmd9");
        assert_eq!(recent[1].command, "cmd8");
        assert_eq!(recent[2].command, "cmd7");
    }

    #[test]
    fn test_output_preview_truncation() {
        let long_output = "a".repeat(200);
        let entry = HistoryEntry::new(
            "agent1".to_string(),
            "test".to_string(),
            RiskLevel::Low,
            100,
            Some(0),
            long_output,
        );

        assert!(entry.output_preview.len() <= 103); // 100 chars + "..."
        assert!(entry.output_preview.ends_with("..."));
    }

    #[test]
    fn test_entry_success_and_failure() {
        let success_entry = create_test_entry("agent1", "ls", RiskLevel::Low, Some(0));
        assert!(success_entry.is_success());
        assert!(!success_entry.is_failure());

        let failed_entry = create_test_entry("agent1", "bad_cmd", RiskLevel::Low, Some(1));
        assert!(!failed_entry.is_success());
        assert!(failed_entry.is_failure());

        let no_exit_entry = create_test_entry("agent1", "running", RiskLevel::Low, None);
        assert!(!no_exit_entry.is_success());
        assert!(!no_exit_entry.is_failure());
    }

    #[test]
    fn test_from_pipeline_item_completed() {
        use super::{PipelineItem, PipelineStage};

        let pipeline_item = PipelineItem::new(
            "test_agent".to_string(),
            "echo hello".to_string(),
            Some("Print hello".to_string()),
            RiskLevel::Low,
        );

        // Set to completed state
        pipeline_item.stage.set(PipelineStage::Completed);
        pipeline_item.exit_code.set(Some(0));
        pipeline_item.output.set(Some("hello\n".to_string()));

        let history_entry = HistoryEntry::from(&pipeline_item);

        assert_eq!(history_entry.agent_id, "test_agent");
        assert_eq!(history_entry.command, "echo hello");
        assert_eq!(history_entry.risk_level, RiskLevel::Low);
        assert_eq!(history_entry.exit_code, Some(0));
        assert!(history_entry.is_success());
        assert_eq!(history_entry.output_preview, "hello\n");
    }

    #[test]
    fn test_from_pipeline_item_failed() {
        use super::{PipelineItem, PipelineStage};

        let pipeline_item = PipelineItem::new(
            "test_agent".to_string(),
            "false".to_string(),
            None,
            RiskLevel::Low,
        );

        // Set to failed state
        pipeline_item.stage.set(PipelineStage::Failed);
        pipeline_item.exit_code.set(Some(1));
        pipeline_item.output.set(Some("command failed".to_string()));

        let history_entry = HistoryEntry::from(&pipeline_item);

        assert_eq!(history_entry.exit_code, Some(1));
        assert!(history_entry.is_failure());
    }

    #[test]
    fn test_from_pipeline_item_cancelled() {
        use super::{PipelineItem, PipelineStage};

        let pipeline_item = PipelineItem::new(
            "test_agent".to_string(),
            "sleep 100".to_string(),
            None,
            RiskLevel::Low,
        );

        // Set to cancelled state
        pipeline_item.stage.set(PipelineStage::Cancelled);
        pipeline_item.output.set(Some("Cancelled by user".to_string()));

        let history_entry = HistoryEntry::from(&pipeline_item);

        // Should infer exit code 130 (SIGINT) for cancelled commands
        assert_eq!(history_entry.exit_code, Some(130));
    }
}
