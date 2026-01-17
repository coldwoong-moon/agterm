//! AgTerm - AI Agent Terminal Orchestrator
//!
//! Main entry point for the application.

use agterm::app::{config::AppConfig, event_loop::App, logging, state::AppState};
use agterm::error::Result;
use agterm::presentation;
use std::path::PathBuf;

/// Command-line arguments
#[derive(Debug)]
struct Args {
    /// Path to configuration file
    config: Option<PathBuf>,

    /// Working directory
    workdir: Option<PathBuf>,

    /// Log level override
    log_level: Option<String>,

    /// Show version and exit
    version: bool,

    /// Show help and exit
    help: bool,
}

impl Args {
    fn parse() -> Self {
        let mut args = Args {
            config: None,
            workdir: None,
            log_level: None,
            version: false,
            help: false,
        };

        let mut iter = std::env::args().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-c" | "--config" => {
                    args.config = iter.next().map(PathBuf::from);
                }
                "-d" | "--workdir" => {
                    args.workdir = iter.next().map(PathBuf::from);
                }
                "-l" | "--log-level" => {
                    args.log_level = iter.next();
                }
                "-v" | "--version" => {
                    args.version = true;
                }
                "-h" | "--help" => {
                    args.help = true;
                }
                _ => {
                    // Ignore unknown arguments for now
                }
            }
        }

        args
    }
}

fn print_help() {
    println!(
        r#"AgTerm - AI Agent Terminal Orchestrator

USAGE:
    agterm [OPTIONS]

OPTIONS:
    -c, --config <FILE>     Path to configuration file
    -d, --workdir <DIR>     Working directory
    -l, --log-level <LEVEL> Log level (trace, debug, info, warn, error)
    -v, --version           Show version information
    -h, --help              Show this help message

ENVIRONMENT VARIABLES:
    AGTERM_LOGGING__LEVEL   Override log level
    AGTERM_PTY__MAX_SESSIONS Override max PTY sessions
    AGTERM_*                Override any config value (use __ for nesting)

CONFIG FILES (in order of precedence):
    1. Command line --config
    2. .agterm/config.toml (local)
    3. ~/.config/agterm/config.toml (user)
    4. /etc/agterm/config.toml (system)

For more information, visit: https://github.com/user/agterm
"#
    );
}

fn print_version() {
    println!("AgTerm v{}", agterm::VERSION);
    println!("AI Agent Terminal Orchestrator");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Handle --help and --version
    if args.help {
        print_help();
        return Ok(());
    }

    if args.version {
        print_version();
        return Ok(());
    }

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        AppConfig::load_with_file(config_path)?
    } else {
        AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config, using defaults: {}", e);
            AppConfig::default()
        })
    };

    // Override log level if specified
    if let Some(level) = args.log_level {
        config.logging.level = level;
    }

    // Initialize logging
    logging::init_logging(&config.logging)?;

    tracing::info!(
        version = %agterm::VERSION,
        "Starting AgTerm"
    );

    // Create application state
    let mut state = AppState::new(config)?;

    // Override working directory if specified
    if let Some(workdir) = args.workdir {
        if workdir.exists() && workdir.is_dir() {
            state.set_cwd(workdir);
        } else {
            tracing::warn!(path = %workdir.display(), "Specified workdir does not exist");
        }
    }

    // Ensure data directories exist
    ensure_directories(&state)?;

    // Install panic hook to restore terminal on panic
    presentation::install_panic_hook();

    // Initialize TUI
    let mut terminal = presentation::init()?;

    // Create and run the application
    let mut app = App::new(state);

    // Run the main event loop
    let result = app.run(&mut terminal).await;

    // Restore terminal
    presentation::restore()?;

    // Handle any errors from the event loop
    if let Err(e) = result {
        tracing::error!(error = %e, "Application error");
        return Err(e);
    }

    tracing::info!("AgTerm shutdown complete");

    Ok(())
}

/// Ensure required directories exist
fn ensure_directories(state: &AppState) -> Result<()> {
    let data_dir = state.config.data_dir();
    let logs_dir = state.config.logs_dir();

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
        tracing::debug!(path = %data_dir.display(), "Created data directory");
    }

    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)?;
        tracing::debug!(path = %logs_dir.display(), "Created logs directory");
    }

    Ok(())
}
