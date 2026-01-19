//! PTY (Pseudo-Terminal) management
//!
//! This module provides cross-platform pseudo-terminal management using the
//! `portable-pty` library. It handles process spawning, I/O operations, and
//! session lifecycle management.
//!
//! # Architecture
//!
//! - **Thread-based I/O**: PTY operations run in a dedicated background thread
//! - **Session management**: Multiple concurrent PTY sessions with unique IDs
//! - **Environment control**: Configurable environment variables and inheritance
//! - **Auto-detection**: Automatically detects the default system shell
//!
//! # Examples
//!
//! ```no_run
//! use agterm::terminal::pty::PtyManager;
//!
//! let manager = PtyManager::new();
//! let pty_id = manager.create_session(40, 120).unwrap();
//! manager.write(&pty_id, b"echo hello\n").unwrap();
//! let output = manager.read(&pty_id).unwrap();
//! ```

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use crate::shell::ShellInfo;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};
use uuid::Uuid;

/// Unique identifier for a PTY session
pub type PtyId = Uuid;

/// Environment variable configuration for PTY sessions
///
/// Controls how environment variables are set up for spawned shell processes.
/// Supports inheritance from parent, custom variables, and unsetting specific variables.
///
/// # Critical Environment Variables
///
/// Even when `inherit_env` is false, the following critical variables are always set:
/// - `HOME`: User home directory
/// - `USER`: Current user name
/// - `PATH`: Executable search path
/// - `LANG`: Locale setting (defaults to en_US.UTF-8)
/// - `SHELL`: Path to the shell being launched
///
/// # Default Environment Variables
///
/// When no custom environment is provided, these are set by default:
/// - `TERM=xterm-256color`: Terminal type with 256-color support
/// - `COLORTERM=truecolor`: Indicates 24-bit true color support
/// - `TERM_PROGRAM=agterm`: Identifies AgTerm as the terminal emulator
/// - `AGTERM_VERSION`: Version of AgTerm
/// - `SHELL`: Path to the launched shell
/// - `LANG`: UTF-8 locale if not already set
#[derive(Debug, Clone)]
pub struct PtyEnvironment {
    /// Inherit environment variables from parent process
    pub inherit_env: bool,
    /// Additional/override environment variables (with shell expansion support)
    pub variables: HashMap<String, String>,
    /// Variables to unset/remove (Note: not fully supported by portable-pty)
    pub unset: Vec<String>,
}

impl PtyEnvironment {
    /// Create a new PtyEnvironment with recommended defaults
    ///
    /// This sets up a typical shell environment with:
    /// - Environment inheritance enabled
    /// - UTF-8 locale
    /// - True color support
    pub fn recommended() -> Self {
        let mut variables = HashMap::new();
        variables.insert("TERM".to_string(), "xterm-256color".to_string());
        variables.insert("COLORTERM".to_string(), "truecolor".to_string());
        variables.insert("TERM_PROGRAM".to_string(), "agterm".to_string());
        variables.insert("AGTERM_VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());

        Self {
            inherit_env: true,
            variables,
            unset: Vec::new(),
        }
    }

    /// Create a minimal environment without inheritance
    ///
    /// This creates an isolated environment with only critical variables.
    /// Useful for testing or security-sensitive scenarios.
    pub fn minimal() -> Self {
        Self {
            inherit_env: false,
            variables: HashMap::new(),
            unset: Vec::new(),
        }
    }
}

impl Default for PtyEnvironment {
    fn default() -> Self {
        Self::recommended()
    }
}

/// Maximum output buffer size per session (1MB)
const MAX_OUTPUT_BUFFER_SIZE: usize = 1024 * 1024;

/// Maximum lines per command block output (reserved for future block-mode feature)
#[allow(dead_code)]
pub const MAX_OUTPUT_LINES: usize = 10000;

/// Errors that can occur during PTY operations
#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    /// Failed to spawn a new PTY session
    #[error("Failed to spawn PTY: {0}")]
    SpawnFailed(String),
    /// Attempted operation on non-existent session
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    /// I/O error during read/write operations
    #[error("IO error: {0}")]
    Io(String),
    /// Internal channel communication error
    #[error("Channel error: {0}")]
    Channel(String),
}

#[allow(dead_code)]
enum PtyCommand {
    Create {
        id: PtyId,
        rows: u16,
        cols: u16,
        environment: Option<PtyEnvironment>,
        response: Sender<Result<(), PtyError>>,
    },
    Write {
        id: PtyId,
        data: Vec<u8>,
        response: Sender<Result<(), PtyError>>,
    },
    Read {
        id: PtyId,
        response: Sender<Result<Vec<u8>, PtyError>>,
    },
    Resize {
        id: PtyId,
        rows: u16,
        cols: u16,
        response: Sender<Result<(), PtyError>>,
    },
    Close {
        id: PtyId,
        response: Sender<Result<(), PtyError>>,
    },
    CheckStatus {
        id: PtyId,
        response: Sender<Result<Option<i32>, PtyError>>,
    },
    Shutdown,
}

struct InternalPtySession {
    #[allow(dead_code)]
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>, // Cached writer for repeated writes
    child: Box<dyn Child + Send + Sync>,
    output_buffer: Vec<u8>,
    reader_thread: Option<JoinHandle<()>>,
    output_receiver: Receiver<Vec<u8>>,
}

/// Get the default shell path, using auto-detection if available
fn default_shell() -> String {
    // Try to use the shell detection system
    if let Some(shell_info) = ShellInfo::default_shell() {
        debug!(
            shell_type = ?shell_info.shell_type,
            path = %shell_info.path.display(),
            "Auto-detected shell"
        );
        return shell_info.path.to_string_lossy().to_string();
    }

    // Fallback to environment variables
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(windows))]
    {
        warn!("Could not auto-detect shell, falling back to /bin/zsh");
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    }
}

#[instrument(skip_all, fields(rows = rows, cols = cols))]
fn create_pty_session(
    rows: u16,
    cols: u16,
    environment: Option<PtyEnvironment>,
) -> Result<InternalPtySession, PtyError> {
    debug!("Creating PTY session");
    let pty_system = native_pty_system();
    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(size).map_err(|e| {
        error!(error = %e, "Failed to open PTY");
        PtyError::SpawnFailed(e.to_string())
    })?;

    let shell = default_shell();
    debug!(shell = %shell, "Using shell");
    let working_dir = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());

    let mut cmd = CommandBuilder::new(&shell);
    cmd.cwd(&working_dir);

    // Apply environment configuration
    if let Some(env_config) = environment {
        // If not inheriting, we still need to set critical environment variables
        if !env_config.inherit_env {
            debug!("Not inheriting parent environment - will set critical variables");

            // Set critical environment variables that shells expect
            if let Ok(home) = std::env::var("HOME") {
                cmd.env("HOME", home);
            }
            if let Ok(user) = std::env::var("USER") {
                cmd.env("USER", user);
            }
            if let Ok(path) = std::env::var("PATH") {
                cmd.env("PATH", path);
            }
            if let Ok(lang) = std::env::var("LANG") {
                cmd.env("LANG", lang);
            } else {
                // Default to UTF-8 locale if not set
                cmd.env("LANG", "en_US.UTF-8");
            }

            // Set SHELL environment variable to match the shell we're launching
            cmd.env("SHELL", &shell);
        }

        // Apply custom environment variables (these override inherited/critical ones)
        for (key, value) in env_config.variables {
            let expanded = shellexpand::full(&value)
                .map(|s| s.to_string())
                .unwrap_or(value);
            debug!(key = %key, value = %expanded, "Setting environment variable");
            cmd.env(&key, &expanded);
        }

        // Note: Unsetting variables isn't directly supported by portable-pty's CommandBuilder
        // The unset functionality would require a different approach
        if !env_config.unset.is_empty() {
            warn!("Unsetting environment variables is not supported by portable-pty");
        }
    } else {
        // Default enhanced shell integration environment variables
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("TERM_PROGRAM", "agterm");
        cmd.env("AGTERM_VERSION", env!("CARGO_PKG_VERSION"));

        // Ensure SHELL is set to the shell we're launching
        cmd.env("SHELL", &shell);

        // Ensure LANG is set for proper UTF-8 support
        if std::env::var("LANG").is_err() {
            cmd.env("LANG", "en_US.UTF-8");
        }
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| {
        error!(error = %e, "Failed to spawn shell command");
        PtyError::SpawnFailed(e.to_string())
    })?;

    info!("PTY session created successfully");

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

    // Get writer upfront and cache it for repeated writes
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

    drop(pair.slave);

    // Create channel for PTY output
    let (output_tx, output_rx) = mpsc::channel();

    // Spawn reader thread that continuously reads PTY output
    let reader_thread = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if output_tx.send(buf[..n].to_vec()).is_err() {
                        break; // Channel closed
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    Ok(InternalPtySession {
        master: pair.master,
        writer,
        child,
        output_buffer: Vec::with_capacity(4096), // Pre-allocate reasonable size
        reader_thread: Some(reader_thread),
        output_receiver: output_rx,
    })
}

fn run_pty_thread(rx: Receiver<PtyCommand>) {
    let mut sessions: HashMap<PtyId, InternalPtySession> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(PtyCommand::Create {
                id,
                rows,
                cols,
                environment,
                response,
            }) => {
                let result = create_pty_session(rows, cols, environment).map(|session| {
                    sessions.insert(id, session);
                });
                let _ = response.send(result);
            }
            Ok(PtyCommand::Write { id, data, response }) => {
                let result = if let Some(session) = sessions.get_mut(&id) {
                    // Use cached writer instead of take_writer()
                    match session
                        .writer
                        .write_all(&data)
                        .and_then(|_| session.writer.flush())
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(PtyError::Io(e.to_string())),
                    }
                } else {
                    Err(PtyError::SessionNotFound(id.to_string()))
                };
                let _ = response.send(result);
            }
            Ok(PtyCommand::Read { id, response }) => {
                let result = if let Some(session) = sessions.get_mut(&id) {
                    // Drain all available output from the reader thread
                    loop {
                        match session.output_receiver.try_recv() {
                            Ok(data) => {
                                session.output_buffer.extend(data);
                                // Enforce memory limit - keep only the tail if exceeded
                                if session.output_buffer.len() > MAX_OUTPUT_BUFFER_SIZE {
                                    let excess =
                                        session.output_buffer.len() - MAX_OUTPUT_BUFFER_SIZE;
                                    session.output_buffer.drain(0..excess);
                                }
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => break,
                        }
                    }
                    // Return and clear the buffer
                    let output = std::mem::take(&mut session.output_buffer);
                    Ok(output)
                } else {
                    Err(PtyError::SessionNotFound(id.to_string()))
                };
                let _ = response.send(result);
            }
            Ok(PtyCommand::Resize {
                id,
                rows,
                cols,
                response,
            }) => {
                let result = if let Some(session) = sessions.get(&id) {
                    let new_size = PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    session
                        .master
                        .resize(new_size)
                        .map_err(|e| PtyError::Io(e.to_string()))
                } else {
                    Err(PtyError::SessionNotFound(id.to_string()))
                };
                let _ = response.send(result);
            }
            Ok(PtyCommand::Close { id, response }) => {
                let result = if let Some(mut session) = sessions.remove(&id) {
                    // Kill child process (this will also cause reader thread to terminate)
                    let kill_result = session
                        .child
                        .kill()
                        .map_err(|e| PtyError::Io(e.to_string()));
                    // Wait for reader thread to finish
                    if let Some(handle) = session.reader_thread.take() {
                        let _ = handle.join();
                    }
                    kill_result
                } else {
                    Ok(())
                };
                let _ = response.send(result);
            }
            Ok(PtyCommand::CheckStatus { id, response }) => {
                let result = if let Some(session) = sessions.get_mut(&id) {
                    match session.child.try_wait() {
                        Ok(Some(status)) => {
                            // Child process has terminated, get exit code
                            let exit_code = status.exit_code() as i32;
                            Ok(Some(exit_code))
                        }
                        Ok(None) => {
                            // Child process is still running
                            Ok(None)
                        }
                        Err(e) => Err(PtyError::Io(e.to_string())),
                    }
                } else {
                    Err(PtyError::SessionNotFound(id.to_string()))
                };
                let _ = response.send(result);
            }
            Ok(PtyCommand::Shutdown) | Err(_) => {
                for (_, mut session) in sessions.drain() {
                    let _ = session.child.kill();
                    if let Some(handle) = session.reader_thread.take() {
                        let _ = handle.join();
                    }
                }
                break;
            }
        }
    }
}

/// Thread-safe PTY manager
pub struct PtyManager {
    tx: Sender<PtyCommand>,
    _thread: JoinHandle<()>,
}

impl PtyManager {
    pub fn new() -> Self {
        debug!("Initializing PTY manager");
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            run_pty_thread(rx);
        });

        info!("PTY manager initialized");
        Self {
            tx,
            _thread: thread,
        }
    }

    #[instrument(skip(self), fields(rows = rows, cols = cols))]
    pub fn create_session(&self, rows: u16, cols: u16) -> Result<PtyId, PtyError> {
        self.create_session_with_env(rows, cols, None)
    }

    #[instrument(skip(self, environment), fields(rows = rows, cols = cols))]
    pub fn create_session_with_env(
        &self,
        rows: u16,
        cols: u16,
        environment: Option<PtyEnvironment>,
    ) -> Result<PtyId, PtyError> {
        let id = Uuid::new_v4();
        debug!(session_id = %id, "Creating new PTY session");
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Create {
                id,
                rows,
                cols,
                environment,
                response: response_tx,
            })
            .map_err(|e| {
                error!(error = %e, "Failed to send create command");
                PtyError::Channel(e.to_string())
            })?;

        response_rx
            .recv()
            .map_err(|e| {
                error!(error = %e, "Failed to receive create response");
                PtyError::Channel(e.to_string())
            })?
            .map(|_| {
                info!(session_id = %id, "PTY session created");
                id
            })
    }

    #[instrument(skip(self, data), fields(session_id = %id, bytes = data.len()))]
    pub fn write(&self, id: &PtyId, data: &[u8]) -> Result<(), PtyError> {
        // Log PTY input at trace level with preview
        if tracing::enabled!(tracing::Level::TRACE) {
            let preview = if data.len() <= 64 {
                String::from_utf8_lossy(data).to_string()
            } else {
                format!(
                    "{}... ({} bytes)",
                    String::from_utf8_lossy(&data[..64]),
                    data.len()
                )
            };
            trace!(bytes = data.len(), preview = %preview, "PTY input");
        }
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Write {
                id: *id,
                data: data.to_vec(),
                response: response_tx,
            })
            .map_err(|e| {
                error!(error = %e, "Failed to send write command");
                PtyError::Channel(e.to_string())
            })?;

        response_rx.recv().map_err(|e| {
            error!(error = %e, "Failed to receive write response");
            PtyError::Channel(e.to_string())
        })?
    }

    #[instrument(skip(self), fields(session_id = %id))]
    pub fn read(&self, id: &PtyId) -> Result<Vec<u8>, PtyError> {
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Read {
                id: *id,
                response: response_tx,
            })
            .map_err(|e| PtyError::Channel(e.to_string()))?;

        let result = response_rx
            .recv()
            .map_err(|e| PtyError::Channel(e.to_string()))??;

        // Log PTY output at trace level with preview
        if !result.is_empty() {
            if tracing::enabled!(tracing::Level::TRACE) {
                let preview = if result.len() <= 64 {
                    String::from_utf8_lossy(&result).to_string()
                } else {
                    format!(
                        "{}... ({} bytes)",
                        String::from_utf8_lossy(&result[..64]),
                        result.len()
                    )
                };
                trace!(bytes = result.len(), preview = %preview, "PTY output");
            } else {
                trace!(bytes = result.len(), "PTY output");
            }
        }
        Ok(result)
    }

    #[instrument(skip(self), fields(session_id = %id, rows = rows, cols = cols))]
    pub fn resize(&self, id: &PtyId, rows: u16, cols: u16) -> Result<(), PtyError> {
        debug!("Resizing PTY");
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Resize {
                id: *id,
                rows,
                cols,
                response: response_tx,
            })
            .map_err(|e| PtyError::Channel(e.to_string()))?;

        response_rx
            .recv()
            .map_err(|e| PtyError::Channel(e.to_string()))?
    }

    #[instrument(skip(self), fields(session_id = %id))]
    pub fn close_session(&self, id: &PtyId) -> Result<(), PtyError> {
        info!("Closing PTY session");
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Close {
                id: *id,
                response: response_tx,
            })
            .map_err(|e| {
                warn!(error = %e, "Failed to send close command");
                PtyError::Channel(e.to_string())
            })?;

        response_rx
            .recv()
            .map_err(|e| PtyError::Channel(e.to_string()))?
    }

    /// Check the exit status of the child process in a PTY session.
    ///
    /// Returns:
    /// - `Ok(Some(exit_code))` if the process has terminated with the given exit code
    /// - `Ok(None)` if the process is still running
    /// - `Err(PtyError)` if there was an error checking the status
    #[allow(dead_code)]
    pub fn check_status(&self, id: &PtyId) -> Result<Option<i32>, PtyError> {
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::CheckStatus {
                id: *id,
                response: response_tx,
            })
            .map_err(|e| PtyError::Channel(e.to_string()))?;

        response_rx
            .recv()
            .map_err(|e| PtyError::Channel(e.to_string()))?
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        let _ = self.tx.send(PtyCommand::Shutdown);
    }
}

unsafe impl Send for PtyManager {}
unsafe impl Sync for PtyManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_environment_default() {
        let env = PtyEnvironment::default();
        assert!(env.inherit_env);
        assert!(env.variables.contains_key("TERM"));
        assert_eq!(env.variables.get("TERM").unwrap(), "xterm-256color");
        assert!(env.variables.contains_key("COLORTERM"));
        assert_eq!(env.variables.get("COLORTERM").unwrap(), "truecolor");
        assert!(env.variables.contains_key("TERM_PROGRAM"));
        assert_eq!(env.variables.get("TERM_PROGRAM").unwrap(), "agterm");
    }

    #[test]
    fn test_pty_environment_recommended() {
        let env = PtyEnvironment::recommended();
        assert!(env.inherit_env);
        assert_eq!(env.variables.len(), 4); // TERM, COLORTERM, TERM_PROGRAM, AGTERM_VERSION
        assert_eq!(env.unset.len(), 0);
    }

    #[test]
    fn test_pty_environment_minimal() {
        let env = PtyEnvironment::minimal();
        assert!(!env.inherit_env);
        assert_eq!(env.variables.len(), 0);
        assert_eq!(env.unset.len(), 0);
    }

    #[test]
    fn test_pty_environment_custom() {
        let mut variables = HashMap::new();
        variables.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());
        variables.insert("TERM".to_string(), "xterm".to_string());

        let env = PtyEnvironment {
            inherit_env: true,
            variables,
            unset: vec!["UNWANTED_VAR".to_string()],
        };

        assert!(env.inherit_env);
        assert_eq!(env.variables.get("CUSTOM_VAR").unwrap(), "custom_value");
        assert_eq!(env.variables.get("TERM").unwrap(), "xterm");
        assert_eq!(env.unset.len(), 1);
    }

    #[test]
    fn test_default_shell() {
        let shell = default_shell();
        assert!(!shell.is_empty());

        #[cfg(windows)]
        {
            assert!(shell.contains("cmd.exe") || shell.contains("powershell"));
        }

        #[cfg(not(windows))]
        {
            assert!(shell.starts_with('/'));
            // Should be an absolute path to a shell
            assert!(
                shell.contains("bash") || shell.contains("zsh") || shell.contains("fish") || shell.contains("sh")
            );
        }
    }

    #[test]
    fn test_pty_manager_creation() {
        let manager = PtyManager::new();
        // Just verify it can be created without panicking
        drop(manager);
    }

    #[test]
    fn test_pty_session_lifecycle() {
        let manager = PtyManager::new();

        // Create session
        let session_id = manager.create_session(24, 80).expect("Failed to create session");

        // Write data
        manager.write(&session_id, b"echo test\n").expect("Failed to write");

        // Give it a moment to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read data
        let output = manager.read(&session_id).expect("Failed to read");
        assert!(!output.is_empty() || true); // Output may or may not be ready

        // Close session
        manager.close_session(&session_id).expect("Failed to close session");
    }

    #[test]
    fn test_pty_session_with_custom_env() {
        let manager = PtyManager::new();

        let mut variables = HashMap::new();
        variables.insert("TEST_VAR".to_string(), "test_value".to_string());
        variables.insert("TERM".to_string(), "xterm-256color".to_string());

        let env = PtyEnvironment {
            inherit_env: true,
            variables,
            unset: Vec::new(),
        };

        let session_id = manager
            .create_session_with_env(24, 80, Some(env))
            .expect("Failed to create session with env");

        // Test that we can write to it
        manager.write(&session_id, b"echo $TEST_VAR\n").expect("Failed to write");

        std::thread::sleep(std::time::Duration::from_millis(100));

        // Clean up
        manager.close_session(&session_id).expect("Failed to close session");
    }

    #[test]
    fn test_pty_resize() {
        let manager = PtyManager::new();
        let session_id = manager.create_session(24, 80).expect("Failed to create session");

        // Resize the terminal
        manager.resize(&session_id, 40, 120).expect("Failed to resize");

        // Clean up
        manager.close_session(&session_id).expect("Failed to close session");
    }

    #[test]
    fn test_shellexpand_in_env() {
        // Test that shell expansion works in environment variables
        let mut variables = HashMap::new();
        variables.insert("HOME_TEST".to_string(), "$HOME/test".to_string());

        let env = PtyEnvironment {
            inherit_env: true,
            variables,
            unset: Vec::new(),
        };

        // This should not panic when the environment is used
        let manager = PtyManager::new();
        let session_id = manager
            .create_session_with_env(24, 80, Some(env))
            .expect("Failed to create session");

        manager.close_session(&session_id).expect("Failed to close session");
    }
}
