//! PTY (Pseudo-Terminal) management

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use tracing::{debug, error, info, instrument, trace, warn};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use uuid::Uuid;

pub type PtyId = Uuid;

/// Maximum output buffer size per session (1MB)
const MAX_OUTPUT_BUFFER_SIZE: usize = 1024 * 1024;

/// Maximum lines per command block output
pub const MAX_OUTPUT_LINES: usize = 10000;

#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("Failed to spawn PTY: {0}")]
    SpawnFailed(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Channel error: {0}")]
    Channel(String),
}

enum PtyCommand {
    Create {
        id: PtyId,
        rows: u16,
        cols: u16,
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

fn default_shell() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    }
}

#[instrument(skip_all, fields(rows = rows, cols = cols))]
fn create_pty_session(rows: u16, cols: u16) -> Result<InternalPtySession, PtyError> {
    debug!("Creating PTY session");
    let pty_system = native_pty_system();
    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| {
            error!(error = %e, "Failed to open PTY");
            PtyError::SpawnFailed(e.to_string())
        })?;

    let shell = default_shell();
    debug!(shell = %shell, "Using shell");
    let working_dir = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());

    let mut cmd = CommandBuilder::new(&shell);
    cmd.cwd(&working_dir);
    cmd.env("TERM", "xterm-256color");

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| {
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
                response,
            }) => {
                let result = create_pty_session(rows, cols).map(|session| {
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
        let id = Uuid::new_v4();
        debug!(session_id = %id, "Creating new PTY session");
        let (response_tx, response_rx) = mpsc::channel();

        self.tx
            .send(PtyCommand::Create {
                id,
                rows,
                cols,
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
        trace!(bytes = data.len(), "PTY write");
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

        response_rx
            .recv()
            .map_err(|e| {
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

        if !result.is_empty() {
            trace!(bytes = result.len(), "PTY read");
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
