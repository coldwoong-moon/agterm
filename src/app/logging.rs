//! Logging Initialization
//!
//! Configures tracing-subscriber for structured logging.

use crate::app::config::LoggingConfig;
use crate::error::Result;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, time::SystemTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initialize the logging system based on configuration
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Create the subscriber based on format
    match config.format.as_str() {
        "json" => init_json_logging(config, env_filter)?,
        "compact" => init_compact_logging(config, env_filter)?,
        _ => init_pretty_logging(config, env_filter)?,
    }

    tracing::info!(
        target: "agterm::init",
        level = %config.level,
        format = %config.format,
        "Logging initialized"
    );

    Ok(())
}

fn init_pretty_logging(config: &LoggingConfig, env_filter: EnvFilter) -> Result<()> {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(config.file_line)
        .with_line_number(config.file_line)
        .with_ansi(true);

    let fmt_layer = if config.timestamps {
        fmt_layer.with_timer(SystemTime::default()).boxed()
    } else {
        fmt_layer.without_time().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn init_compact_logging(config: &LoggingConfig, env_filter: EnvFilter) -> Result<()> {
    let fmt_layer = fmt::layer()
        .compact()
        .with_target(true)
        .with_level(true)
        .with_file(config.file_line)
        .with_line_number(config.file_line)
        .with_ansi(true);

    let fmt_layer = if config.timestamps {
        fmt_layer.with_timer(SystemTime::default()).boxed()
    } else {
        fmt_layer.without_time().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn init_json_logging(config: &LoggingConfig, env_filter: EnvFilter) -> Result<()> {
    let fmt_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_file(config.file_line)
        .with_line_number(config.file_line)
        .with_current_span(true);

    let fmt_layer = if config.timestamps {
        fmt_layer.with_timer(SystemTime::default()).boxed()
    } else {
        fmt_layer.without_time().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn parse_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" | "warning" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// Initialize logging with defaults (for quick start or tests)
pub fn init_default_logging() -> Result<()> {
    let config = LoggingConfig::default();
    init_logging(&config)
}

/// Initialize logging with a specific level
pub fn init_with_level(level: &str) -> Result<()> {
    let config = LoggingConfig {
        level: level.to_string(),
        ..Default::default()
    };
    init_logging(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level() {
        assert_eq!(parse_level("trace"), Level::TRACE);
        assert_eq!(parse_level("DEBUG"), Level::DEBUG);
        assert_eq!(parse_level("Info"), Level::INFO);
        assert_eq!(parse_level("WARN"), Level::WARN);
        assert_eq!(parse_level("error"), Level::ERROR);
        assert_eq!(parse_level("unknown"), Level::INFO);
    }
}
