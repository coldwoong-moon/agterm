//! Terminal statistics and analytics system
//!
//! Provides comprehensive tracking and analysis of terminal usage including:
//! - Command execution statistics with timing and success rates
//! - Session tracking with metrics (commands, bytes, errors)
//! - Daily, weekly, and monthly aggregation
//! - Productivity scoring based on usage patterns
//! - Serialization to JSON and CSV export
//! - Trend analysis and command frequency tracking

use chrono::{Datelike, DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during statistics operations
#[derive(Debug, Error)]
pub enum StatisticsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid date range")]
    InvalidDateRange,

    #[error("No data available for the specified period")]
    NoDataAvailable,
}

/// Statistics for a single command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandStat {
    /// Command name/text
    pub command: String,
    /// Number of times executed
    pub count: u64,
    /// Total execution duration across all invocations
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    /// Last time this command was used
    pub last_used: DateTime<Utc>,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
}

impl CommandStat {
    /// Create a new command statistic
    pub fn new(command: String) -> Self {
        Self {
            command,
            count: 0,
            total_duration: Duration::zero(),
            last_used: Utc::now(),
            success_count: 0,
            failure_count: 0,
        }
    }

    /// Record a command execution
    pub fn record(&mut self, duration: Duration, success: bool) {
        self.count += 1;
        self.total_duration = self.total_duration + duration;
        self.last_used = Utc::now();
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    /// Get average execution duration
    pub fn average_duration(&self) -> Duration {
        if self.count == 0 {
            Duration::zero()
        } else {
            Duration::milliseconds(self.total_duration.num_milliseconds() / self.count as i64)
        }
    }

    /// Get success rate as percentage (0.0-100.0)
    pub fn success_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.count as f64) * 100.0
        }
    }
}

/// Statistics for a terminal session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStat {
    /// Unique session identifier
    pub session_id: String,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Session end time (None if still active)
    pub end_time: Option<DateTime<Utc>>,
    /// Number of commands executed in this session
    pub commands_executed: u64,
    /// Total bytes read from PTY
    pub bytes_read: u64,
    /// Total bytes written to PTY
    pub bytes_written: u64,
    /// Number of errors encountered
    pub errors_count: u64,
}

impl SessionStat {
    /// Create a new session statistic
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            start_time: Utc::now(),
            end_time: None,
            commands_executed: 0,
            bytes_read: 0,
            bytes_written: 0,
            errors_count: 0,
        }
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end.signed_duration_since(self.start_time)
    }

    /// Check if session is still active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// End the session
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }
}

/// Daily aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyStat {
    /// Date for this statistic
    pub date: NaiveDate,
    /// Total number of sessions started
    pub total_sessions: u64,
    /// Total number of commands executed
    pub total_commands: u64,
    /// Total active time across all sessions
    #[serde(with = "duration_serde")]
    pub active_time: Duration,
    /// Most used commands (command, count)
    pub most_used_commands: Vec<(String, u64)>,
}

impl DailyStat {
    /// Create a new daily statistic
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            total_sessions: 0,
            total_commands: 0,
            active_time: Duration::zero(),
            most_used_commands: Vec::new(),
        }
    }

    /// Get average commands per session
    pub fn avg_commands_per_session(&self) -> f64 {
        if self.total_sessions == 0 {
            0.0
        } else {
            self.total_commands as f64 / self.total_sessions as f64
        }
    }

    /// Get average session duration
    pub fn avg_session_duration(&self) -> Duration {
        if self.total_sessions == 0 {
            Duration::zero()
        } else {
            Duration::milliseconds(self.active_time.num_milliseconds() / self.total_sessions as i64)
        }
    }
}

/// Weekly summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeeklySummary {
    /// Start date of the week
    pub period_start: NaiveDate,
    /// End date of the week
    pub period_end: NaiveDate,
    /// Total sessions in the week
    pub total_sessions: u64,
    /// Total commands in the week
    pub total_commands: u64,
    /// Total active time in the week
    #[serde(with = "duration_serde")]
    pub total_active_time: Duration,
    /// Daily breakdown
    pub daily_breakdown: Vec<DailyStat>,
    /// Command trends (command, change from previous week)
    pub command_trends: Vec<(String, i64)>,
}

/// Monthly summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlySummary {
    /// Start date of the month
    pub period_start: NaiveDate,
    /// End date of the month
    pub period_end: NaiveDate,
    /// Total sessions in the month
    pub total_sessions: u64,
    /// Total commands in the month
    pub total_commands: u64,
    /// Total active time in the month
    #[serde(with = "duration_serde")]
    pub total_active_time: Duration,
    /// Daily breakdown
    pub daily_breakdown: Vec<DailyStat>,
    /// Command trends (command, change from previous month)
    pub command_trends: Vec<(String, i64)>,
}

/// Main statistics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsCollector {
    /// All command statistics
    commands: HashMap<String, CommandStat>,
    /// All session statistics
    sessions: HashMap<String, SessionStat>,
    /// Daily statistics
    daily_stats: HashMap<NaiveDate, DailyStat>,
    /// Path to statistics file
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl StatisticsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            sessions: HashMap::new(),
            daily_stats: HashMap::new(),
            file_path: None,
        }
    }

    /// Record a command execution
    pub fn record_command(&mut self, command: String, duration: Duration, success: bool) {
        // Update command stats
        let stat = self
            .commands
            .entry(command.clone())
            .or_insert_with(|| CommandStat::new(command));
        stat.record(duration, success);

        // Update daily stats
        let today = Utc::now().date_naive();
        let daily = self
            .daily_stats
            .entry(today)
            .or_insert_with(|| DailyStat::new(today));
        daily.total_commands += 1;

        tracing::debug!("Recorded command execution");
    }

    /// Record session start
    pub fn record_session_start(&mut self, session_id: String) {
        let stat = SessionStat::new(session_id.clone());
        self.sessions.insert(session_id, stat);

        // Update daily stats
        let today = Utc::now().date_naive();
        let daily = self
            .daily_stats
            .entry(today)
            .or_insert_with(|| DailyStat::new(today));
        daily.total_sessions += 1;

        tracing::debug!("Recorded session start");
    }

    /// Record session end
    pub fn record_session_end(&mut self, session_id: &str) -> Result<(), StatisticsError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| StatisticsError::SessionNotFound(session_id.to_string()))?;

        if session.is_active() {
            let duration = session.duration();
            session.end();

            // Update daily stats
            let today = Utc::now().date_naive();
            if let Some(daily) = self.daily_stats.get_mut(&today) {
                daily.active_time = daily.active_time + duration;
            }

            tracing::debug!("Recorded session end: {}", session_id);
        }

        Ok(())
    }

    /// Record bytes read from PTY
    pub fn record_bytes_read(&mut self, session_id: &str, bytes: u64) -> Result<(), StatisticsError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| StatisticsError::SessionNotFound(session_id.to_string()))?;
        session.bytes_read += bytes;
        Ok(())
    }

    /// Record bytes written to PTY
    pub fn record_bytes_written(&mut self, session_id: &str, bytes: u64) -> Result<(), StatisticsError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| StatisticsError::SessionNotFound(session_id.to_string()))?;
        session.bytes_written += bytes;
        Ok(())
    }

    /// Record an error
    pub fn record_error(&mut self, session_id: &str) -> Result<(), StatisticsError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| StatisticsError::SessionNotFound(session_id.to_string()))?;
        session.errors_count += 1;
        Ok(())
    }

    /// Save statistics to file
    pub fn save_to_file(&self, path: PathBuf) -> Result<(), StatisticsError> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        tracing::info!("Statistics saved to {:?}", path);
        Ok(())
    }

    /// Load statistics from file
    pub fn load_from_file(path: PathBuf) -> Result<Self, StatisticsError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let json = std::fs::read_to_string(&path)?;
        let mut stats: StatisticsCollector = serde_json::from_str(&json)?;
        stats.file_path = Some(path);

        tracing::info!("Statistics loaded from {:?}", stats.file_path);
        Ok(stats)
    }

    /// Get the default statistics file path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("statistics.json")
    }

    /// Auto-save if file path is set
    pub fn auto_save(&self) -> Result<(), StatisticsError> {
        if let Some(path) = &self.file_path {
            self.save_to_file(path.clone())?;
        }
        Ok(())
    }

    /// Export to CSV format
    pub fn export_csv(&self) -> Result<String, StatisticsError> {
        let mut csv = String::new();

        // Command statistics
        csv.push_str("Command Statistics\n");
        csv.push_str("Command,Count,Total Duration (ms),Average Duration (ms),Success Count,Failure Count,Success Rate (%),Last Used\n");

        let mut commands: Vec<_> = self.commands.values().collect();
        commands.sort_by(|a, b| b.count.cmp(&a.count));

        for cmd in commands {
            csv.push_str(&format!(
                "\"{}\",{},{},{},{},{},{:.2},{}\n",
                cmd.command.replace('\"', "\"\""),
                cmd.count,
                cmd.total_duration.num_milliseconds(),
                cmd.average_duration().num_milliseconds(),
                cmd.success_count,
                cmd.failure_count,
                cmd.success_rate(),
                cmd.last_used.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        // Session statistics
        csv.push_str("\nSession Statistics\n");
        csv.push_str("Session ID,Start Time,End Time,Duration (s),Commands,Bytes Read,Bytes Written,Errors\n");

        for session in self.sessions.values() {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                session.session_id,
                session.start_time.format("%Y-%m-%d %H:%M:%S"),
                session
                    .end_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Active".to_string()),
                session.duration().num_seconds(),
                session.commands_executed,
                session.bytes_read,
                session.bytes_written,
                session.errors_count
            ));
        }

        // Daily statistics
        csv.push_str("\nDaily Statistics\n");
        csv.push_str("Date,Sessions,Commands,Active Time (min),Avg Commands/Session,Avg Session Duration (min)\n");

        let mut daily: Vec<_> = self.daily_stats.values().collect();
        daily.sort_by(|a, b| b.date.cmp(&a.date));

        for day in daily {
            csv.push_str(&format!(
                "{},{},{},{:.2},{:.2},{:.2}\n",
                day.date,
                day.total_sessions,
                day.total_commands,
                day.active_time.num_minutes() as f64,
                day.avg_commands_per_session(),
                day.avg_session_duration().num_minutes() as f64
            ));
        }

        Ok(csv)
    }

    /// Clear all statistics
    pub fn clear(&mut self) {
        self.commands.clear();
        self.sessions.clear();
        self.daily_stats.clear();
        tracing::info!("Statistics cleared");
    }
}

impl Default for StatisticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics analyzer for querying and analyzing collected data
#[derive(Debug)]
pub struct StatisticsAnalyzer<'a> {
    collector: &'a StatisticsCollector,
}

impl<'a> StatisticsAnalyzer<'a> {
    /// Create a new analyzer from a collector
    pub fn new(collector: &'a StatisticsCollector) -> Self {
        Self { collector }
    }

    /// Get top N most used commands
    pub fn get_top_commands(&self, limit: usize) -> Vec<&CommandStat> {
        let mut commands: Vec<_> = self.collector.commands.values().collect();
        commands.sort_by(|a, b| {
            b.count.cmp(&a.count).then_with(|| a.command.cmp(&b.command))
        });
        commands.into_iter().take(limit).collect()
    }

    /// Get frequency statistics for a specific command
    pub fn get_command_frequency(&self, command: &str) -> Option<&CommandStat> {
        self.collector.commands.get(command)
    }

    /// Get daily statistics for a specific date
    pub fn get_daily_stats(&self, date: NaiveDate) -> Option<&DailyStat> {
        self.collector.daily_stats.get(&date)
    }

    /// Get weekly summary for the current week
    pub fn get_weekly_summary(&self) -> Result<WeeklySummary, StatisticsError> {
        let today = Utc::now().date_naive();
        let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let week_end = week_start + Duration::days(6);

        self.get_weekly_summary_for_range(week_start, week_end)
    }

    /// Get weekly summary for a specific date range
    fn get_weekly_summary_for_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<WeeklySummary, StatisticsError> {
        if start > end {
            return Err(StatisticsError::InvalidDateRange);
        }

        let mut total_sessions = 0;
        let mut total_commands = 0;
        let mut total_active_time = Duration::zero();
        let mut daily_breakdown = Vec::new();

        let mut current = start;
        while current <= end {
            if let Some(daily) = self.collector.daily_stats.get(&current) {
                total_sessions += daily.total_sessions;
                total_commands += daily.total_commands;
                total_active_time = total_active_time + daily.active_time;
                daily_breakdown.push(daily.clone());
            } else {
                daily_breakdown.push(DailyStat::new(current));
            }
            current = current.succ_opt().ok_or(StatisticsError::InvalidDateRange)?;
        }

        // Calculate command trends (comparing with previous week)
        let command_trends = self.calculate_command_trends(start, end)?;

        Ok(WeeklySummary {
            period_start: start,
            period_end: end,
            total_sessions,
            total_commands,
            total_active_time,
            daily_breakdown,
            command_trends,
        })
    }

    /// Get monthly summary for the current month
    pub fn get_monthly_summary(&self) -> Result<MonthlySummary, StatisticsError> {
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .ok_or(StatisticsError::InvalidDateRange)?;
        let month_end = if today.month() == 12 {
            NaiveDate::from_ymd_opt(today.year(), 12, 31)
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
                .and_then(|d| d.pred_opt())
        }
        .ok_or(StatisticsError::InvalidDateRange)?;

        self.get_monthly_summary_for_range(month_start, month_end)
    }

    /// Get monthly summary for a specific date range
    fn get_monthly_summary_for_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<MonthlySummary, StatisticsError> {
        if start > end {
            return Err(StatisticsError::InvalidDateRange);
        }

        let mut total_sessions = 0;
        let mut total_commands = 0;
        let mut total_active_time = Duration::zero();
        let mut daily_breakdown = Vec::new();

        let mut current = start;
        while current <= end {
            if let Some(daily) = self.collector.daily_stats.get(&current) {
                total_sessions += daily.total_sessions;
                total_commands += daily.total_commands;
                total_active_time = total_active_time + daily.active_time;
                daily_breakdown.push(daily.clone());
            } else {
                daily_breakdown.push(DailyStat::new(current));
            }
            current = current.succ_opt().ok_or(StatisticsError::InvalidDateRange)?;
        }

        // Calculate command trends (comparing with previous month)
        let command_trends = self.calculate_command_trends(start, end)?;

        Ok(MonthlySummary {
            period_start: start,
            period_end: end,
            total_sessions,
            total_commands,
            total_active_time,
            daily_breakdown,
            command_trends,
        })
    }

    /// Calculate command trends by comparing periods
    fn calculate_command_trends(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<(String, i64)>, StatisticsError> {
        // Count commands in current period
        let mut current_counts: HashMap<String, i64> = HashMap::new();

        for (cmd, stat) in &self.collector.commands {
            let cmd_date = stat.last_used.date_naive();
            if cmd_date >= start && cmd_date <= end {
                *current_counts.entry(cmd.clone()).or_insert(0) += stat.count as i64;
            }
        }

        // Calculate period length and get previous period
        let period_length = (end - start).num_days();
        let prev_start = start - Duration::days(period_length + 1);
        let prev_end = start - Duration::days(1);

        // Count commands in previous period
        let mut prev_counts: HashMap<String, i64> = HashMap::new();

        for (cmd, stat) in &self.collector.commands {
            let cmd_date = stat.last_used.date_naive();
            if cmd_date >= prev_start && cmd_date <= prev_end {
                *prev_counts.entry(cmd.clone()).or_insert(0) += stat.count as i64;
            }
        }

        // Calculate trends
        let mut trends: Vec<(String, i64)> = current_counts
            .iter()
            .map(|(cmd, &current)| {
                let prev = prev_counts.get(cmd).copied().unwrap_or(0);
                (cmd.clone(), current - prev)
            })
            .collect();

        // Sort by absolute change (descending)
        trends.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));

        Ok(trends)
    }

    /// Get session statistics for a specific session
    pub fn get_session_stats(&self, session_id: &str) -> Option<&SessionStat> {
        self.collector.sessions.get(session_id)
    }

    /// Get average session duration across all sessions
    pub fn get_average_session_duration(&self) -> Duration {
        if self.collector.sessions.is_empty() {
            return Duration::zero();
        }

        let total: i64 = self
            .collector
            .sessions
            .values()
            .map(|s| s.duration().num_milliseconds())
            .sum();

        Duration::milliseconds(total / self.collector.sessions.len() as i64)
    }

    /// Calculate productivity score (0.0-100.0)
    ///
    /// Based on:
    /// - Command execution frequency
    /// - Success rate
    /// - Session duration consistency
    /// - Active time ratio
    pub fn get_productivity_score(&self) -> f64 {
        if self.collector.daily_stats.is_empty() {
            return 0.0;
        }

        // Calculate average commands per day
        let total_days = self.collector.daily_stats.len() as f64;
        let total_commands: u64 = self
            .collector
            .daily_stats
            .values()
            .map(|d| d.total_commands)
            .sum();
        let avg_commands_per_day = total_commands as f64 / total_days;

        // Calculate overall success rate
        let total_successes: u64 = self.collector.commands.values().map(|c| c.success_count).sum();
        let total_executions: u64 = self.collector.commands.values().map(|c| c.count).sum();
        let success_rate = if total_executions > 0 {
            (total_successes as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        // Calculate session consistency (lower variance = more consistent)
        let avg_session_duration = self.get_average_session_duration().num_seconds() as f64;
        let variance: f64 = self
            .collector
            .sessions
            .values()
            .map(|s| {
                let diff = s.duration().num_seconds() as f64 - avg_session_duration;
                diff * diff
            })
            .sum::<f64>()
            / self.collector.sessions.len() as f64;
        let std_dev = variance.sqrt();
        let consistency_score = if avg_session_duration > 0.0 {
            (100.0 - (std_dev / avg_session_duration * 100.0).min(100.0)).max(0.0)
        } else {
            0.0
        };

        // Weighted average of factors
        let command_score = (avg_commands_per_day.min(100.0) / 100.0) * 30.0;
        let success_score = (success_rate / 100.0) * 40.0;
        let consistency = (consistency_score / 100.0) * 30.0;

        (command_score + success_score + consistency).min(100.0)
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&SessionStat> {
        self.collector
            .sessions
            .values()
            .filter(|s| s.is_active())
            .collect()
    }

    /// Get total number of unique commands
    pub fn get_unique_command_count(&self) -> usize {
        self.collector.commands.len()
    }

    /// Get total execution count across all commands
    pub fn get_total_executions(&self) -> u64 {
        self.collector.commands.values().map(|c| c.count).sum()
    }

    /// Get most productive day
    pub fn get_most_productive_day(&self) -> Option<&DailyStat> {
        self.collector
            .daily_stats
            .values()
            .max_by_key(|d| d.total_commands)
    }

    /// Get least productive day (excluding zero-command days)
    pub fn get_least_productive_day(&self) -> Option<&DailyStat> {
        self.collector
            .daily_stats
            .values()
            .filter(|d| d.total_commands > 0)
            .min_by_key(|d| d.total_commands)
    }
}

// Custom serialization for Duration
mod duration_serde {
    use chrono::Duration;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.num_milliseconds().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        Ok(Duration::milliseconds(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::TempDir;

    #[test]
    fn test_command_stat_creation() {
        let stat = CommandStat::new("ls -la".to_string());
        assert_eq!(stat.command, "ls -la");
        assert_eq!(stat.count, 0);
        assert_eq!(stat.success_count, 0);
        assert_eq!(stat.failure_count, 0);
    }

    #[test]
    fn test_command_stat_recording() {
        let mut stat = CommandStat::new("git status".to_string());

        stat.record(Duration::milliseconds(150), true);
        stat.record(Duration::milliseconds(200), true);
        stat.record(Duration::milliseconds(180), false);

        assert_eq!(stat.count, 3);
        assert_eq!(stat.success_count, 2);
        assert_eq!(stat.failure_count, 1);
        assert_eq!(stat.total_duration.num_milliseconds(), 530);
        assert_eq!(stat.average_duration().num_milliseconds(), 176);
        assert!((stat.success_rate() - 200.0 / 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_session_stat_creation() {
        let session = SessionStat::new("session-123".to_string());
        assert_eq!(session.session_id, "session-123");
        assert!(session.is_active());
        assert_eq!(session.commands_executed, 0);
    }

    #[test]
    fn test_session_stat_end() {
        let mut session = SessionStat::new("session-456".to_string());
        assert!(session.is_active());

        session.end();
        assert!(!session.is_active());
        assert!(session.end_time.is_some());
    }

    #[test]
    fn test_daily_stat_calculations() {
        let mut daily = DailyStat::new(Utc::now().date_naive());
        daily.total_sessions = 5;
        daily.total_commands = 50;
        daily.active_time = Duration::minutes(150);

        assert_eq!(daily.avg_commands_per_session(), 10.0);
        assert_eq!(daily.avg_session_duration().num_minutes(), 30);
    }

    #[test]
    fn test_collector_record_command() {
        let mut collector = StatisticsCollector::new();

        collector.record_command(
            "echo test".to_string(),
            Duration::milliseconds(100),
            true,
        );
        collector.record_command(
            "echo test".to_string(),
            Duration::milliseconds(120),
            true,
        );

        assert_eq!(collector.commands.len(), 1);
        let stat = collector.commands.get("echo test").unwrap();
        assert_eq!(stat.count, 2);
        assert_eq!(stat.success_count, 2);
    }

    #[test]
    fn test_collector_session_lifecycle() {
        let mut collector = StatisticsCollector::new();

        collector.record_session_start("session-1".to_string());
        assert_eq!(collector.sessions.len(), 1);

        let result = collector.record_bytes_read("session-1", 1024);
        assert!(result.is_ok());

        let result = collector.record_bytes_written("session-1", 512);
        assert!(result.is_ok());

        let result = collector.record_error("session-1");
        assert!(result.is_ok());

        let session = collector.sessions.get("session-1").unwrap();
        assert_eq!(session.bytes_read, 1024);
        assert_eq!(session.bytes_written, 512);
        assert_eq!(session.errors_count, 1);

        let result = collector.record_session_end("session-1");
        assert!(result.is_ok());

        let session = collector.sessions.get("session-1").unwrap();
        assert!(!session.is_active());
    }

    #[test]
    fn test_collector_session_not_found() {
        let mut collector = StatisticsCollector::new();

        let result = collector.record_bytes_read("nonexistent", 100);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StatisticsError::SessionNotFound(_)
        ));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("stats.json");

        let mut collector = StatisticsCollector::new();
        collector.record_command("test".to_string(), Duration::milliseconds(100), true);
        collector.record_session_start("session-1".to_string());

        collector.save_to_file(path.clone()).unwrap();
        assert!(path.exists());

        let loaded = StatisticsCollector::load_from_file(path).unwrap();
        assert_eq!(loaded.commands.len(), 1);
        assert_eq!(loaded.sessions.len(), 1);
    }

    #[test]
    fn test_export_csv() {
        let mut collector = StatisticsCollector::new();
        collector.record_command("ls".to_string(), Duration::milliseconds(50), true);
        collector.record_session_start("session-1".to_string());

        let csv = collector.export_csv().unwrap();
        assert!(csv.contains("Command Statistics"));
        assert!(csv.contains("Session Statistics"));
        assert!(csv.contains("Daily Statistics"));
        assert!(csv.contains("ls"));
    }

    #[test]
    fn test_analyzer_top_commands() {
        let mut collector = StatisticsCollector::new();
        collector.record_command("git".to_string(), Duration::milliseconds(100), true);
        collector.record_command("git".to_string(), Duration::milliseconds(100), true);
        collector.record_command("git".to_string(), Duration::milliseconds(100), true);
        collector.record_command("ls".to_string(), Duration::milliseconds(50), true);
        collector.record_command("pwd".to_string(), Duration::milliseconds(30), true);

        let analyzer = StatisticsAnalyzer::new(&collector);
        let top = analyzer.get_top_commands(2);

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].command, "git");
        assert_eq!(top[0].count, 3);
        assert_eq!(top[1].command, "ls");
    }

    #[test]
    fn test_analyzer_command_frequency() {
        let mut collector = StatisticsCollector::new();
        collector.record_command("test".to_string(), Duration::milliseconds(100), true);

        let analyzer = StatisticsAnalyzer::new(&collector);
        let freq = analyzer.get_command_frequency("test");

        assert!(freq.is_some());
        assert_eq!(freq.unwrap().count, 1);

        let none = analyzer.get_command_frequency("nonexistent");
        assert!(none.is_none());
    }

    #[test]
    fn test_analyzer_average_session_duration() {
        let mut collector = StatisticsCollector::new();

        // Create sessions with known durations
        let session1 = SessionStat {
            session_id: "s1".to_string(),
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            end_time: Some(Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap()),
            commands_executed: 5,
            bytes_read: 1000,
            bytes_written: 500,
            errors_count: 0,
        };

        let session2 = SessionStat {
            session_id: "s2".to_string(),
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap(),
            end_time: Some(Utc.with_ymd_and_hms(2024, 1, 1, 11, 20, 0).unwrap()),
            commands_executed: 10,
            bytes_read: 2000,
            bytes_written: 1000,
            errors_count: 1,
        };

        collector.sessions.insert("s1".to_string(), session1);
        collector.sessions.insert("s2".to_string(), session2);

        let analyzer = StatisticsAnalyzer::new(&collector);
        let avg = analyzer.get_average_session_duration();

        // Average of 10 minutes and 20 minutes = 15 minutes
        assert_eq!(avg.num_minutes(), 15);
    }

    #[test]
    fn test_analyzer_productivity_score() {
        let mut collector = StatisticsCollector::new();

        // Add some commands with good success rate
        collector.record_command("cmd1".to_string(), Duration::milliseconds(100), true);
        collector.record_command("cmd2".to_string(), Duration::milliseconds(100), true);
        collector.record_command("cmd3".to_string(), Duration::milliseconds(100), true);
        collector.record_command("cmd4".to_string(), Duration::milliseconds(100), false);

        // Add sessions
        collector.record_session_start("s1".to_string());
        collector.record_session_start("s2".to_string());

        let analyzer = StatisticsAnalyzer::new(&collector);
        let score = analyzer.get_productivity_score();

        assert!(score >= 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_analyzer_unique_command_count() {
        let mut collector = StatisticsCollector::new();
        collector.record_command("cmd1".to_string(), Duration::milliseconds(100), true);
        collector.record_command("cmd1".to_string(), Duration::milliseconds(100), true);
        collector.record_command("cmd2".to_string(), Duration::milliseconds(100), true);

        let analyzer = StatisticsAnalyzer::new(&collector);
        assert_eq!(analyzer.get_unique_command_count(), 2);
        assert_eq!(analyzer.get_total_executions(), 3);
    }

    #[test]
    fn test_analyzer_active_sessions() {
        let mut collector = StatisticsCollector::new();
        collector.record_session_start("active1".to_string());
        collector.record_session_start("active2".to_string());
        collector.record_session_start("inactive".to_string());
        collector.record_session_end("inactive").unwrap();

        let analyzer = StatisticsAnalyzer::new(&collector);
        let active = analyzer.get_active_sessions();

        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_analyzer_most_productive_day() {
        let mut collector = StatisticsCollector::new();

        let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();

        let mut day1 = DailyStat::new(date1);
        day1.total_commands = 50;

        let mut day2 = DailyStat::new(date2);
        day2.total_commands = 100;

        collector.daily_stats.insert(date1, day1);
        collector.daily_stats.insert(date2, day2);

        let analyzer = StatisticsAnalyzer::new(&collector);
        let most_productive = analyzer.get_most_productive_day().unwrap();

        assert_eq!(most_productive.date, date2);
        assert_eq!(most_productive.total_commands, 100);
    }

    #[test]
    fn test_weekly_summary() {
        let mut collector = StatisticsCollector::new();

        // Add some daily stats
        for i in 0..7 {
            let date = Utc::now().date_naive() - Duration::days(i);
            let mut daily = DailyStat::new(date);
            daily.total_sessions = 2;
            daily.total_commands = 20;
            daily.active_time = Duration::minutes(60);
            collector.daily_stats.insert(date, daily);
        }

        let analyzer = StatisticsAnalyzer::new(&collector);
        let summary = analyzer.get_weekly_summary();

        assert!(summary.is_ok());
        let summary = summary.unwrap();
        assert_eq!(summary.daily_breakdown.len(), 7);
        assert!(summary.total_sessions > 0);
    }

    #[test]
    fn test_monthly_summary() {
        let mut collector = StatisticsCollector::new();

        // Add daily stats for current month
        let today = Utc::now().date_naive();
        for i in 0..10 {
            let date = today - Duration::days(i);
            if date.month() == today.month() {
                let mut daily = DailyStat::new(date);
                daily.total_sessions = 3;
                daily.total_commands = 30;
                daily.active_time = Duration::minutes(90);
                collector.daily_stats.insert(date, daily);
            }
        }

        let analyzer = StatisticsAnalyzer::new(&collector);
        let summary = analyzer.get_monthly_summary();

        assert!(summary.is_ok());
        let summary = summary.unwrap();
        assert!(summary.total_sessions > 0);
    }

    #[test]
    fn test_clear_statistics() {
        let mut collector = StatisticsCollector::new();
        collector.record_command("test".to_string(), Duration::milliseconds(100), true);
        collector.record_session_start("session-1".to_string());

        assert!(!collector.commands.is_empty());
        assert!(!collector.sessions.is_empty());

        collector.clear();

        assert!(collector.commands.is_empty());
        assert!(collector.sessions.is_empty());
        assert!(collector.daily_stats.is_empty());
    }

    #[test]
    fn test_command_stat_edge_cases() {
        let mut stat = CommandStat::new("test".to_string());

        // Zero count should give zero average
        assert_eq!(stat.average_duration(), Duration::zero());
        assert_eq!(stat.success_rate(), 0.0);

        // Single execution
        stat.record(Duration::milliseconds(100), true);
        assert_eq!(stat.average_duration().num_milliseconds(), 100);
        assert_eq!(stat.success_rate(), 100.0);

        // All failures
        stat.record(Duration::milliseconds(50), false);
        stat.record(Duration::milliseconds(50), false);
        assert!((stat.success_rate() - 100.0 / 3.0).abs() < 0.0001);
    }
}
