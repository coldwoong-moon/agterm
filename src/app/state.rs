//! Application State
//!
//! Global application state management.

use crate::app::config::AppConfig;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Application running state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningState {
    /// Application is starting up
    Starting,
    /// Application is running normally
    Running,
    /// Application is shutting down
    ShuttingDown,
    /// Application has stopped
    Stopped,
}

/// Main application state
#[derive(Debug)]
pub struct AppState {
    /// Unique session ID for this run
    pub session_id: Uuid,

    /// Application configuration
    pub config: AppConfig,

    /// Current running state
    pub running_state: RunningState,

    /// Current working directory
    pub working_dir: std::path::PathBuf,

    /// Active view (for TUI)
    pub active_view: ActiveView,

    /// Whether the application should quit
    pub should_quit: bool,
}

/// Active TUI view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    /// Main view with task tree and terminals
    #[default]
    Main,
    /// Graph view (F4)
    Graph,
    /// MCP panel (F5)
    Mcp,
    /// Archive browser (F6)
    Archive,
    /// Help screen (F1)
    Help,
}

impl AppState {
    /// Create a new application state with the given configuration
    pub fn new(config: AppConfig) -> Result<Self> {
        let working_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));

        Ok(Self {
            session_id: Uuid::new_v4(),
            config,
            running_state: RunningState::Starting,
            working_dir,
            active_view: ActiveView::Main,
            should_quit: false,
        })
    }

    /// Transition to running state
    pub fn start(&mut self) {
        self.running_state = RunningState::Running;
        tracing::info!(
            session_id = %self.session_id,
            working_dir = %self.working_dir.display(),
            "Application started"
        );
    }

    /// Request application shutdown
    pub fn request_shutdown(&mut self) {
        if self.running_state == RunningState::Running {
            self.running_state = RunningState::ShuttingDown;
            self.should_quit = true;
            tracing::info!("Shutdown requested");
        }
    }

    /// Mark application as stopped
    pub fn stop(&mut self) {
        self.running_state = RunningState::Stopped;
        tracing::info!("Application stopped");
    }

    /// Check if application is running
    pub fn is_running(&self) -> bool {
        self.running_state == RunningState::Running
    }

    /// Switch to a different view
    pub fn switch_view(&mut self, view: ActiveView) {
        tracing::debug!(from = ?self.active_view, to = ?view, "Switching view");
        self.active_view = view;
    }

    /// Get the current working directory
    pub fn cwd(&self) -> &std::path::Path {
        &self.working_dir
    }

    /// Change working directory
    pub fn set_cwd(&mut self, path: std::path::PathBuf) {
        tracing::info!(
            from = %self.working_dir.display(),
            to = %path.display(),
            "Changing working directory"
        );
        self.working_dir = path;
    }
}

/// Thread-safe wrapper for AppState
pub type SharedAppState = Arc<RwLock<AppState>>;

/// Create a new shared application state
pub fn create_shared_state(config: AppConfig) -> Result<SharedAppState> {
    let state = AppState::new(config)?;
    Ok(Arc::new(RwLock::new(state)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_lifecycle() {
        let config = AppConfig::default();
        let mut state = AppState::new(config).unwrap();

        assert_eq!(state.running_state, RunningState::Starting);
        assert!(!state.is_running());

        state.start();
        assert_eq!(state.running_state, RunningState::Running);
        assert!(state.is_running());

        state.request_shutdown();
        assert_eq!(state.running_state, RunningState::ShuttingDown);
        assert!(!state.is_running());
        assert!(state.should_quit);

        state.stop();
        assert_eq!(state.running_state, RunningState::Stopped);
    }

    #[test]
    fn test_view_switching() {
        let config = AppConfig::default();
        let mut state = AppState::new(config).unwrap();

        assert_eq!(state.active_view, ActiveView::Main);

        state.switch_view(ActiveView::Graph);
        assert_eq!(state.active_view, ActiveView::Graph);

        state.switch_view(ActiveView::Archive);
        assert_eq!(state.active_view, ActiveView::Archive);
    }
}
