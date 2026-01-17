//! PTY Session Management
//!
//! Manages a single pseudo-terminal session with async I/O.

use crate::error::{PtyError, PtyResult};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// PTY session identifier
pub type PtyId = Uuid;

/// PTY session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyState {
    /// Session is starting
    Starting,
    /// Session is running
    Running,
    /// Session has exited
    Exited,
    /// Session encountered an error
    Error,
}

/// Configuration for creating a new PTY session
#[derive(Debug, Clone)]
pub struct PtySessionConfig {
    /// Shell command to execute
    pub shell: String,
    /// Arguments to pass to the shell
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: PathBuf,
    /// Environment variables to set
    pub env: Vec<(String, String)>,
    /// Terminal size
    pub size: PtySize,
}

impl Default for PtySessionConfig {
    fn default() -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        Self {
            shell,
            args: Vec::new(),
            working_dir,
            env: Vec::new(),
            size: PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            },
        }
    }
}

/// A single PTY session
pub struct PtySession {
    /// Unique session ID
    pub id: PtyId,
    /// Current state
    state: Arc<Mutex<PtyState>>,
    /// Master PTY handle
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    /// Child process
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
    /// Current terminal size
    size: Arc<Mutex<PtySize>>,
    /// Configuration used to create this session
    pub config: PtySessionConfig,
}

impl PtySession {
    /// Create a new PTY session
    pub fn new(config: PtySessionConfig) -> PtyResult<Self> {
        let pty_system = native_pty_system();

        // Create the PTY pair
        let pair = pty_system.openpty(config.size.clone()).map_err(|e| {
            PtyError::SpawnFailed {
                reason: format!("Failed to open PTY: {}", e),
            }
        })?;

        // Build the command
        let mut cmd = CommandBuilder::new(&config.shell);
        cmd.args(&config.args);
        cmd.cwd(&config.working_dir);

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Spawn the child process
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            PtyError::SpawnFailed {
                reason: format!("Failed to spawn command: {}", e),
            }
        })?;

        // Drop the slave to avoid blocking reads
        drop(pair.slave);

        Ok(Self {
            id: Uuid::new_v4(),
            state: Arc::new(Mutex::new(PtyState::Running)),
            master: Arc::new(Mutex::new(pair.master)),
            child: Arc::new(Mutex::new(child)),
            size: Arc::new(Mutex::new(config.size.clone())),
            config,
        })
    }

    /// Get the current state
    pub async fn state(&self) -> PtyState {
        *self.state.lock().await
    }

    /// Check if the session is still running
    pub async fn is_running(&self) -> bool {
        self.state().await == PtyState::Running
    }

    /// Resize the terminal
    pub async fn resize(&self, rows: u16, cols: u16) -> PtyResult<()> {
        let new_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let master = self.master.lock().await;
        master.resize(new_size.clone()).map_err(|e| {
            PtyError::ResizeFailed {
                reason: e.to_string(),
            }
        })?;

        *self.size.lock().await = new_size;
        Ok(())
    }

    /// Get the current terminal size
    pub async fn size(&self) -> PtySize {
        self.size.lock().await.clone()
    }

    /// Write data to the PTY (user input)
    pub async fn write(&self, data: &[u8]) -> PtyResult<usize> {
        let mut master = self.master.lock().await;
        let mut writer = master.take_writer().map_err(|e| {
            PtyError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let written = writer.write(data)?;
        writer.flush()?;
        Ok(written)
    }

    /// Write a string to the PTY
    pub async fn write_str(&self, s: &str) -> PtyResult<usize> {
        self.write(s.as_bytes()).await
    }

    /// Read available data from the PTY (blocking)
    pub fn read_blocking(&self, buf: &mut [u8]) -> PtyResult<usize> {
        // This is a synchronous read - use in a blocking task
        let master = self.master.blocking_lock();
        let mut reader = master.try_clone_reader().map_err(|e| {
            PtyError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        match reader.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(PtyError::Io(e)),
        }
    }

    /// Try to get the exit status (non-blocking)
    pub async fn try_wait(&self) -> PtyResult<Option<u32>> {
        let mut child = self.child.lock().await;

        match child.try_wait() {
            Ok(Some(status)) => {
                *self.state.lock().await = PtyState::Exited;
                Ok(Some(status.exit_code()))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                *self.state.lock().await = PtyState::Error;
                Err(PtyError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )))
            }
        }
    }

    /// Kill the child process
    pub async fn kill(&self) -> PtyResult<()> {
        let mut child = self.child.lock().await;
        child.kill().map_err(|e| {
            PtyError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        *self.state.lock().await = PtyState::Exited;
        Ok(())
    }

    /// Get a reader for the PTY output
    pub async fn get_reader(&self) -> PtyResult<Box<dyn Read + Send>> {
        let master = self.master.lock().await;
        master.try_clone_reader().map_err(|e| {
            PtyError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }
}

impl std::fmt::Debug for PtySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySession")
            .field("id", &self.id)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pty_session_creation() {
        let config = PtySessionConfig::default();
        let session = PtySession::new(config);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert!(session.is_running().await);
    }
}
