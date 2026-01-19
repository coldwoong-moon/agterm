//! Logging system initialization and configuration
//!
//! Uses the tracing ecosystem for structured logging with support for:
//! - Environment variable override (AGTERM_LOG)
//! - File output with daily rotation
//! - Console output for development
//! - Module-level log filtering
//! - In-memory log buffer for debug panel

pub mod layers;

pub use layers::LogBuffer;

use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use layers::LogBufferLayer;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Default log level
    pub level: Level,
    /// Output format: "pretty", "json", "compact"
    pub format: LogFormat,
    /// Show timestamps
    pub timestamps: bool,
    /// Show file and line numbers
    pub file_line: bool,
    /// Enable file output
    pub file_output: bool,
    /// Log file directory path
    pub file_path: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Pretty,
            timestamps: true,
            file_line: false,
            file_output: true,
            file_path: None,
        }
    }
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

impl LogFormat {
    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        }
    }
}

/// Get the default log directory path
fn default_log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agterm")
        .join("logs")
}

/// Default log buffer size for debug panel
const DEFAULT_LOG_BUFFER_SIZE: usize = 100;

/// Initialize the logging system
///
/// Returns a `LogBuffer` handle that can be used to read logs in the debug panel.
///
/// # Environment Variables
/// - `AGTERM_LOG`: Override log level (e.g., "agterm=debug,agterm::terminal::pty=trace")
/// - `AGTERM_LOG_LEVEL`: Simple log level override (trace, debug, info, warn, error)
/// - `AGTERM_LOG_PATH`: Override log file directory path
/// - `AGTERM_DEBUG`: Enable debug panel on startup
pub fn init_logging(config: &LoggingConfig) -> LogBuffer {
    // Allow simple level override via AGTERM_LOG_LEVEL
    let effective_level = std::env::var("AGTERM_LOG_LEVEL")
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "trace" => Some(Level::TRACE),
            "debug" => Some(Level::DEBUG),
            "info" => Some(Level::INFO),
            "warn" | "warning" => Some(Level::WARN),
            "error" => Some(Level::ERROR),
            _ => None,
        })
        .unwrap_or(config.level);

    // Build the environment filter
    let env_filter = EnvFilter::try_from_env("AGTERM_LOG").unwrap_or_else(|_| {
        EnvFilter::new(format!("agterm={}", effective_level.as_str().to_lowercase()))
    });

    // Create the console layer
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(config.file_line)
        .with_line_number(config.file_line)
        .with_ansi(true);

    let console_layer = if config.timestamps {
        console_layer.boxed()
    } else {
        console_layer.without_time().boxed()
    };

    // Create file layer if enabled
    let file_layer = if config.file_output {
        // Allow log path override via environment variable
        let log_dir = std::env::var("AGTERM_LOG_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| config.file_path.clone())
            .unwrap_or_else(default_log_dir);

        // Ensure log directory exists
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!(
                "Warning: Failed to create log directory {log_dir:?}: {e}"
            );
            None
        } else {
            eprintln!("AgTerm logs will be written to: {}", log_dir.display());

            let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "agterm.log");

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_span_events(FmtSpan::CLOSE);

            Some(file_layer.boxed())
        }
    } else {
        None
    };

    // Create in-memory log buffer layer for debug panel
    let (log_buffer_layer, log_buffer) = LogBufferLayer::new(DEFAULT_LOG_BUFFER_SIZE);

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .with(log_buffer_layer)
        .init();

    tracing::info!(
        level = %effective_level,
        file_output = config.file_output,
        "AgTerm logging initialized"
    );
    tracing::debug!(
        level = %effective_level,
        format = ?config.format,
        file_output = config.file_output,
        timestamps = config.timestamps,
        file_line = config.file_line,
        "Detailed logging configuration"
    );

    log_buffer
}

/// Parse log level from string
#[allow(dead_code)]
pub fn parse_level(s: &str) -> Level {
    match s.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" | "warning" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("json"), LogFormat::Json);
        assert_eq!(LogFormat::from_str("JSON"), LogFormat::Json);
        assert_eq!(LogFormat::from_str("compact"), LogFormat::Compact);
        assert_eq!(LogFormat::from_str("pretty"), LogFormat::Pretty);
        assert_eq!(LogFormat::from_str("unknown"), LogFormat::Pretty);
    }

    #[test]
    fn test_parse_level() {
        assert_eq!(parse_level("trace"), Level::TRACE);
        assert_eq!(parse_level("DEBUG"), Level::DEBUG);
        assert_eq!(parse_level("info"), Level::INFO);
        assert_eq!(parse_level("warn"), Level::WARN);
        assert_eq!(parse_level("warning"), Level::WARN);
        assert_eq!(parse_level("error"), Level::ERROR);
        assert_eq!(parse_level("unknown"), Level::INFO);
    }

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, LogFormat::Pretty);
        assert!(config.timestamps);
        assert!(!config.file_line);
        assert!(config.file_output);
    }
}
