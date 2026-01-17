//! Compaction Strategies
//!
//! Implements the COMPRESS pattern from Context Engineering:
//! 1. Output Reference (reversible) - store full log, keep reference
//! 2. Summarization (lossy) - AI-based summary
//! 3. Hierarchical Rolling - progressive summarization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Compacted output result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactedOutput {
    /// Key information extracted from output
    pub summary: String,
    /// Last N lines of output
    pub last_lines: Vec<String>,
    /// Path to full log file
    pub full_log_path: PathBuf,
    /// Original size in bytes
    pub original_size: usize,
    /// Compacted size in bytes
    pub compacted_size: usize,
    /// Compaction time
    pub compacted_at: DateTime<Utc>,
}

impl CompactedOutput {
    /// Get compression ratio (0.0 - 1.0)
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            1.0
        } else {
            self.compacted_size as f64 / self.original_size as f64
        }
    }

    /// Format as display string
    pub fn to_display_string(&self) -> String {
        let mut output = self.summary.clone();

        if !self.last_lines.is_empty() {
            output.push_str("\n\n--- Last Output ---\n");
            for line in &self.last_lines {
                output.push_str(line);
                output.push('\n');
            }
        }

        output.push_str(&format!(
            "\n[Full log: {}]",
            self.full_log_path.display()
        ));

        output
    }
}

/// Compaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Number of last lines to keep
    pub last_lines_count: usize,
    /// Log storage directory
    pub log_dir: PathBuf,
    /// Maximum output size before compaction (bytes)
    pub max_output_size: usize,
    /// Enable AI summarization
    pub enable_ai_summary: bool,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            last_lines_count: 10,
            log_dir: PathBuf::from(".agterm/logs"),
            max_output_size: 10 * 1024, // 10KB
            enable_ai_summary: false,
        }
    }
}

/// Output compactor
pub struct Compactor {
    config: CompactionConfig,
}

impl Compactor {
    /// Create a new compactor with configuration
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CompactionConfig::default())
    }

    /// Set log directory
    pub fn with_log_dir(mut self, dir: PathBuf) -> Self {
        self.config.log_dir = dir;
        self
    }

    /// Compact output, saving full log to file
    pub fn compact(&self, output: &str, task_id: &Uuid) -> std::io::Result<CompactedOutput> {
        let original_size = output.len();

        // Save full log
        let log_path = self.save_log(output, task_id)?;

        // Extract last N lines
        let last_lines: Vec<String> = output
            .lines()
            .rev()
            .take(self.config.last_lines_count)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Extract key information
        let summary = self.extract_summary(output);

        let compacted_size = summary.len() + last_lines.iter().map(|s| s.len()).sum::<usize>();

        Ok(CompactedOutput {
            summary,
            last_lines,
            full_log_path: log_path,
            original_size,
            compacted_size,
            compacted_at: Utc::now(),
        })
    }

    /// Save log to file
    fn save_log(&self, output: &str, task_id: &Uuid) -> std::io::Result<PathBuf> {
        // Ensure log directory exists
        std::fs::create_dir_all(&self.config.log_dir)?;

        // Generate filename
        let filename = format!("{}.log", task_id);
        let path = self.config.log_dir.join(filename);

        // Write log
        std::fs::write(&path, output)?;

        Ok(path)
    }

    /// Extract key information from output
    fn extract_summary(&self, output: &str) -> String {
        let mut summary = String::new();

        // Detect exit status
        if let Some(exit_info) = self.detect_exit_status(output) {
            summary.push_str(&exit_info);
            summary.push('\n');
        }

        // Detect errors
        let errors = self.detect_errors(output);
        if !errors.is_empty() {
            summary.push_str("Errors:\n");
            for error in errors.iter().take(5) {
                summary.push_str(&format!("  - {}\n", error));
            }
            if errors.len() > 5 {
                summary.push_str(&format!("  ... and {} more\n", errors.len() - 5));
            }
        }

        // Detect warnings
        let warnings = self.detect_warnings(output);
        if !warnings.is_empty() {
            summary.push_str(&format!("Warnings: {} total\n", warnings.len()));
        }

        // Detect success indicators
        if let Some(success_info) = self.detect_success(output) {
            summary.push_str(&success_info);
            summary.push('\n');
        }

        if summary.is_empty() {
            summary = format!("Output: {} lines, {} bytes",
                output.lines().count(),
                output.len()
            );
        }

        summary
    }

    /// Detect exit status from output
    fn detect_exit_status(&self, output: &str) -> Option<String> {
        let lower = output.to_lowercase();

        if lower.contains("exit code: 0") || lower.contains("exited with 0") {
            Some("Exit: 0 (success)".to_string())
        } else if let Some(pos) = lower.find("exit code:") {
            let rest = &output[pos..];
            let code: String = rest
                .chars()
                .skip(10)
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            if !code.is_empty() {
                Some(format!("Exit: {}", code))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Detect errors in output
    fn detect_errors(&self, output: &str) -> Vec<String> {
        let mut errors = Vec::new();

        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("error:")
                || lower.contains("error[")
                || lower.contains("fatal:")
                || lower.contains("exception:")
                || lower.contains("panic:")
            {
                errors.push(line.trim().to_string());
            }
        }

        errors
    }

    /// Detect warnings in output
    fn detect_warnings(&self, output: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("warning:") || lower.contains("warn:") {
                warnings.push(line.trim().to_string());
            }
        }

        warnings
    }

    /// Detect success indicators
    fn detect_success(&self, output: &str) -> Option<String> {
        let lower = output.to_lowercase();

        if lower.contains("successfully") {
            Some("Status: Success".to_string())
        } else if lower.contains("compiled") && lower.contains("modules") {
            Some("Status: Compiled".to_string())
        } else if lower.contains("test") && lower.contains("passed") {
            Some("Status: Tests passed".to_string())
        } else if lower.contains("done") || lower.contains("complete") {
            Some("Status: Done".to_string())
        } else {
            None
        }
    }

    /// Check if output needs compaction
    pub fn needs_compaction(&self, output: &str) -> bool {
        output.len() > self.config.max_output_size
    }
}

/// AI Summary request (for future AI integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRequest {
    /// Content to summarize
    pub content: String,
    /// Maximum summary length (tokens)
    pub max_tokens: usize,
    /// Summary type
    pub summary_type: SummaryType,
}

/// Summary type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SummaryType {
    /// Session summary
    Session,
    /// Task output summary
    TaskOutput,
    /// Error summary
    Error,
    /// Daily/weekly rolling summary
    Rolling,
}

/// AI Summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResponse {
    /// Generated summary
    pub summary: String,
    /// Token count
    pub token_count: usize,
    /// Key points extracted
    pub key_points: Vec<String>,
    /// Suggested tags
    pub suggested_tags: Vec<String>,
}

/// Summary prompt template
pub fn generate_summary_prompt(request: &SummaryRequest) -> String {
    match request.summary_type {
        SummaryType::Session => {
            format!(
                r#"다음 터미널 세션 로그를 요약해주세요.
- 실행된 주요 명령어
- 성공/실패 여부 및 핵심 결과
- 발생한 에러 메시지 (있는 경우)
- 다음 세션에서 참고할 만한 정보

로그:
{}

최대 {} 토큰으로 요약해주세요."#,
                request.content, request.max_tokens
            )
        }
        SummaryType::TaskOutput => {
            format!(
                r#"다음 명령어 출력을 한 문장으로 요약해주세요.
- 성공 여부
- 핵심 결과

출력:
{}

최대 {} 토큰으로 요약해주세요."#,
                request.content, request.max_tokens
            )
        }
        SummaryType::Error => {
            format!(
                r#"다음 에러 메시지를 분석하고 해결 방법을 제안해주세요.

에러:
{}

최대 {} 토큰으로 답변해주세요."#,
                request.content, request.max_tokens
            )
        }
        SummaryType::Rolling => {
            format!(
                r#"다음 여러 세션 요약들을 하나의 상위 요약으로 통합해주세요.
- 주요 작업 트렌드
- 반복되는 패턴
- 주요 성과 및 문제점

세션 요약들:
{}

최대 {} 토큰으로 요약해주세요."#,
                request.content, request.max_tokens
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();

        assert_eq!(config.last_lines_count, 10);
        assert!(!config.enable_ai_summary);
    }

    #[test]
    fn test_extract_errors() {
        let compactor = Compactor::with_defaults();
        let output = r#"
Building...
error: cannot find module 'foo'
error: type mismatch
warning: unused variable
Done.
"#;

        let errors = compactor.detect_errors(output);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_extract_warnings() {
        let compactor = Compactor::with_defaults();
        let output = r#"
Compiling...
warning: unused import
warning: deprecated function
Finished.
"#;

        let warnings = compactor.detect_warnings(output);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_detect_success() {
        let compactor = Compactor::with_defaults();

        assert!(compactor.detect_success("Build completed successfully").is_some());
        assert!(compactor.detect_success("All tests passed").is_some());
        assert!(compactor.detect_success("random text").is_none());
    }

    #[test]
    fn test_needs_compaction() {
        let config = CompactionConfig {
            max_output_size: 100,
            ..Default::default()
        };
        let compactor = Compactor::new(config);

        assert!(!compactor.needs_compaction("short output"));
        assert!(compactor.needs_compaction(&"x".repeat(200)));
    }

    #[test]
    fn test_compression_ratio() {
        let output = CompactedOutput {
            summary: String::new(),
            last_lines: vec![],
            full_log_path: PathBuf::from("/tmp/test.log"),
            original_size: 1000,
            compacted_size: 250,
            compacted_at: Utc::now(),
        };

        assert_eq!(output.compression_ratio(), 0.25);
    }
}
