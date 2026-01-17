//! PTY Pool Management
//!
//! Manages a pool of PTY sessions with configurable limits and lifecycle management.

use crate::error::{PtyError, PtyResult};
use crate::infrastructure::pty::parser::AnsiParser;
use crate::infrastructure::pty::session::{PtyId, PtySession, PtySessionConfig};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Configuration for the PTY pool
#[derive(Debug, Clone)]
pub struct PtyPoolConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Default terminal size (rows, cols)
    pub default_size: (u16, u16),
    /// Timeout for idle sessions in seconds (0 = no timeout)
    pub idle_timeout_secs: u64,
}

impl Default for PtyPoolConfig {
    fn default() -> Self {
        Self {
            max_sessions: 32,
            default_size: (24, 80),
            idle_timeout_secs: 0,
        }
    }
}

/// A managed PTY session with its associated parser
pub struct ManagedSession {
    /// The PTY session
    pub session: PtySession,
    /// ANSI parser for this session
    pub parser: AnsiParser,
    /// Reader for PTY output
    reader: Option<Arc<Mutex<Box<dyn Read + Send>>>>,
    /// Session label/name
    pub label: String,
    /// Creation timestamp
    pub created_at: std::time::Instant,
    /// Last activity timestamp
    pub last_activity: std::time::Instant,
}

impl ManagedSession {
    /// Create a new managed session
    fn new(session: PtySession, size: (u16, u16), label: String) -> Self {
        let now = std::time::Instant::now();
        Self {
            session,
            parser: AnsiParser::new(size.1 as usize, size.0 as usize),
            reader: None,
            label,
            created_at: now,
            last_activity: now,
        }
    }

    /// Initialize the reader
    pub async fn init_reader(&mut self) -> PtyResult<()> {
        if self.reader.is_none() {
            let reader = self.session.get_reader().await?;
            self.reader = Some(Arc::new(Mutex::new(reader)));
        }
        Ok(())
    }

    /// Get the reader reference
    #[must_use]
    pub fn reader(&self) -> Option<&Arc<Mutex<Box<dyn Read + Send>>>> {
        self.reader.as_ref()
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// Check if the session is idle
    #[must_use]
    pub fn idle_duration(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }
}

impl std::fmt::Debug for ManagedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSession")
            .field("id", &self.session.id)
            .field("label", &self.label)
            .field("created_at", &self.created_at)
            .field("last_activity", &self.last_activity)
            .finish_non_exhaustive()
    }
}

/// PTY Pool - manages multiple PTY sessions
pub struct PtyPool {
    /// Pool configuration
    config: PtyPoolConfig,
    /// Active sessions
    sessions: RwLock<HashMap<PtyId, ManagedSession>>,
    /// Currently focused session ID
    focused_id: RwLock<Option<PtyId>>,
    /// Session order (for tab-like navigation)
    session_order: RwLock<Vec<PtyId>>,
}

impl PtyPool {
    /// Create a new PTY pool with the given configuration
    #[must_use]
    pub fn new(config: PtyPoolConfig) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            focused_id: RwLock::new(None),
            session_order: RwLock::new(Vec::new()),
        }
    }

    /// Create a new PTY pool with default configuration
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(PtyPoolConfig::default())
    }

    /// Get the pool configuration
    pub fn config(&self) -> &PtyPoolConfig {
        &self.config
    }

    /// Get the number of active sessions
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Check if the pool is at capacity
    pub async fn is_full(&self) -> bool {
        self.session_count().await >= self.config.max_sessions
    }

    /// Spawn a new PTY session
    pub async fn spawn(&self, session_config: PtySessionConfig, label: String) -> PtyResult<PtyId> {
        // Check pool capacity
        if self.is_full().await {
            return Err(PtyError::PoolExhausted {
                max: self.config.max_sessions,
                current: self.session_count().await,
            });
        }

        // Create the session
        let size = (session_config.size.rows, session_config.size.cols);
        let session = PtySession::new(session_config)?;
        let id = session.id;

        // Create managed session
        let mut managed = ManagedSession::new(session, size, label);

        // Initialize the reader
        managed.init_reader().await?;

        // Add to pool
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(id, managed);
        }

        // Add to order list
        {
            let mut order = self.session_order.write().await;
            order.push(id);
        }

        // Set as focused if this is the first session
        {
            let mut focused = self.focused_id.write().await;
            if focused.is_none() {
                *focused = Some(id);
            }
        }

        tracing::info!(pty_id = %id, "PTY session spawned");
        Ok(id)
    }

    /// Spawn a session with default configuration
    pub async fn spawn_default(&self, label: String) -> PtyResult<PtyId> {
        let mut config = PtySessionConfig::default();
        config.size.rows = self.config.default_size.0;
        config.size.cols = self.config.default_size.1;
        self.spawn(config, label).await
    }

    /// Spawn a session with a specific command
    pub async fn spawn_with_command(
        &self,
        command: &str,
        args: &[String],
        working_dir: &std::path::Path,
    ) -> PtyResult<PtyId> {
        use portable_pty::PtySize;

        let config = PtySessionConfig {
            shell: command.to_string(),
            args: args.to_vec(),
            working_dir: working_dir.to_path_buf(),
            env: Vec::new(),
            size: PtySize {
                rows: self.config.default_size.0,
                cols: self.config.default_size.1,
                pixel_width: 0,
                pixel_height: 0,
            },
        };

        let label = format!("{} {}", command, args.join(" "));
        self.spawn(config, label).await
    }

    /// Get a reference to a managed session
    pub fn get_session(&self, id: &PtyId) -> Option<&ManagedSession> {
        // Note: This is a blocking call, use carefully
        futures::executor::block_on(async {
            let sessions = self.sessions.read().await;
            // We can't return a reference to data behind RwLock easily
            // This is a design limitation - the caller should use with_session instead
            None
        })
    }

    /// Get a reference to a session by ID
    pub async fn get(&self, id: &PtyId) -> Option<PtyId> {
        let sessions = self.sessions.read().await;
        if sessions.contains_key(id) {
            Some(*id)
        } else {
            None
        }
    }

    /// Execute a function with access to a session
    pub async fn with_session<F, R>(&self, id: &PtyId, f: F) -> PtyResult<R>
    where
        F: FnOnce(&ManagedSession) -> R,
    {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| PtyError::NotFound { id: id.to_string() })?;
        Ok(f(session))
    }

    /// Execute a mutable function with access to a session
    pub async fn with_session_mut<F, R>(&self, id: &PtyId, f: F) -> PtyResult<R>
    where
        F: FnOnce(&mut ManagedSession) -> R,
    {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| PtyError::NotFound { id: id.to_string() })?;
        Ok(f(session))
    }

    /// Kill and remove a session
    pub async fn kill(&self, id: &PtyId) -> PtyResult<()> {
        // Remove from sessions
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(id)
        };

        let Some(managed) = session else {
            return Err(PtyError::NotFound { id: id.to_string() });
        };

        // Kill the PTY process
        managed.session.kill().await?;

        // Remove from order
        {
            let mut order = self.session_order.write().await;
            order.retain(|i| i != id);
        }

        // Update focus if this was the focused session
        {
            let mut focused = self.focused_id.write().await;
            if *focused == Some(*id) {
                let order = self.session_order.read().await;
                *focused = order.first().copied();
            }
        }

        tracing::info!(pty_id = %id, "PTY session killed");
        Ok(())
    }

    /// Remove a session that has exited
    pub async fn remove_exited(&self, id: &PtyId) -> PtyResult<()> {
        // Remove from sessions
        {
            let mut sessions = self.sessions.write().await;
            sessions.remove(id);
        }

        // Remove from order
        {
            let mut order = self.session_order.write().await;
            order.retain(|i| i != id);
        }

        // Update focus if this was the focused session
        {
            let mut focused = self.focused_id.write().await;
            if *focused == Some(*id) {
                let order = self.session_order.read().await;
                *focused = order.first().copied();
            }
        }

        tracing::debug!(pty_id = %id, "Exited PTY session removed");
        Ok(())
    }

    /// Get the currently focused session ID
    pub async fn focused_id(&self) -> Option<PtyId> {
        *self.focused_id.read().await
    }

    /// Set the focused session
    pub async fn set_focus(&self, id: PtyId) -> PtyResult<()> {
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(&id) {
            return Err(PtyError::NotFound { id: id.to_string() });
        }

        *self.focused_id.write().await = Some(id);
        tracing::debug!(pty_id = %id, "Focus changed");
        Ok(())
    }

    /// Focus the next session in order
    pub async fn focus_next(&self) -> Option<PtyId> {
        let order = self.session_order.read().await;
        if order.is_empty() {
            return None;
        }

        let mut focused = self.focused_id.write().await;
        let current_idx = focused
            .and_then(|id| order.iter().position(|i| *i == id))
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % order.len();
        let next_id = order[next_idx];
        *focused = Some(next_id);
        Some(next_id)
    }

    /// Focus the previous session in order
    pub async fn focus_prev(&self) -> Option<PtyId> {
        let order = self.session_order.read().await;
        if order.is_empty() {
            return None;
        }

        let mut focused = self.focused_id.write().await;
        let current_idx = focused
            .and_then(|id| order.iter().position(|i| *i == id))
            .unwrap_or(0);

        let prev_idx = if current_idx == 0 {
            order.len() - 1
        } else {
            current_idx - 1
        };
        let prev_id = order[prev_idx];
        *focused = Some(prev_id);
        Some(prev_id)
    }

    /// Get all session IDs in order
    pub async fn session_ids(&self) -> Vec<PtyId> {
        self.session_order.read().await.clone()
    }

    /// Get session info for all sessions
    pub async fn session_info(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let order = self.session_order.read().await;
        let focused = self.focused_id.read().await;

        order
            .iter()
            .filter_map(|id| {
                sessions.get(id).map(|s| SessionInfo {
                    id: *id,
                    label: s.label.clone(),
                    is_focused: *focused == Some(*id),
                    created_at: s.created_at,
                    last_activity: s.last_activity,
                })
            })
            .collect()
    }

    /// Resize all sessions
    pub async fn resize_all(&self, rows: u16, cols: u16) -> PtyResult<()> {
        let mut sessions = self.sessions.write().await;
        for (id, managed) in sessions.iter_mut() {
            if let Err(e) = managed.session.resize(rows, cols).await {
                tracing::warn!(pty_id = %id, error = %e, "Failed to resize session");
            }
            managed.parser.resize(cols as usize, rows as usize);
        }
        Ok(())
    }

    /// Check for and remove exited sessions
    pub async fn cleanup_exited(&self) -> Vec<PtyId> {
        let mut exited = Vec::new();

        // First, find exited sessions
        {
            let sessions = self.sessions.read().await;
            for (id, managed) in sessions.iter() {
                if let Ok(Some(_)) = managed.session.try_wait().await {
                    exited.push(*id);
                }
            }
        }

        // Then remove them
        for id in &exited {
            let _ = self.remove_exited(id).await;
        }

        exited
    }

    /// Kill all sessions
    pub async fn kill_all(&self) -> PtyResult<()> {
        let ids: Vec<PtyId> = self.session_ids().await;
        for id in ids {
            if let Err(e) = self.kill(&id).await {
                tracing::warn!(pty_id = %id, error = %e, "Failed to kill session");
            }
        }
        Ok(())
    }

    /// Write data to the focused session
    pub async fn write_to_focused(&self, data: &[u8]) -> PtyResult<usize> {
        let focused_id = self.focused_id().await.ok_or_else(|| PtyError::NotFound {
            id: "no focused session".to_string(),
        })?;

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&focused_id)
            .ok_or_else(|| PtyError::NotFound {
                id: focused_id.to_string(),
            })?;

        session.session.write(data).await
    }

    /// Get screen data and label for a session (cloned)
    pub async fn get_screen_data(
        &self,
        id: &PtyId,
    ) -> PtyResult<(crate::infrastructure::pty::parser::TerminalScreen, String)> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| PtyError::NotFound { id: id.to_string() })?;

        let screen = session.parser.screen().clone();
        let label = session.label.clone();
        Ok((screen, label))
    }

    /// Read and process PTY output for all sessions
    pub async fn read_all_outputs(&self) -> HashMap<PtyId, String> {
        let mut outputs = HashMap::new();
        let mut sessions = self.sessions.write().await;
        for (id, managed) in sessions.iter_mut() {
            // Clone the reader Arc to avoid borrow conflicts
            let reader = managed.reader().cloned();
            if let Some(reader) = reader {
                if let Ok(mut reader_guard) = reader.try_lock() {
                    let mut buf = [0u8; 4096];
                    match reader_guard.read(&mut buf) {
                        Ok(0) => {}
                        Ok(n) => {
                            // Now we can safely mutate managed
                            drop(reader_guard);
                            let data = String::from_utf8_lossy(&buf[..n]).to_string();
                            managed.parser.process(&buf[..n]);
                            managed.touch();
                            outputs.insert(*id, data);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            tracing::warn!(pty_id = %id, error = %e, "Error reading PTY");
                        }
                    }
                }
            }
        }
        outputs
    }

    /// Synchronous wrapper for `spawn_with_command`
    pub fn spawn_with_command_sync(
        &self,
        command: &str,
        args: &[String],
        working_dir: &std::path::Path,
    ) -> PtyResult<PtyId> {
        futures::executor::block_on(self.spawn_with_command(command, args, working_dir))
    }

    /// Synchronous wrapper for kill
    pub fn kill_sync(&self, id: &PtyId) -> PtyResult<()> {
        futures::executor::block_on(self.kill(id))
    }
}

impl std::fmt::Debug for PtyPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyPool")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Information about a session
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session ID
    pub id: PtyId,
    /// Session label
    pub label: String,
    /// Whether this session is focused
    pub is_focused: bool,
    /// Creation time
    pub created_at: std::time::Instant,
    /// Last activity time
    pub last_activity: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_pty::PtySize;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = PtyPool::with_defaults();
        assert_eq!(pool.session_count().await, 0);
        assert!(!pool.is_full().await);
    }

    #[tokio::test]
    async fn test_spawn_session() {
        let pool = PtyPool::with_defaults();

        let config = PtySessionConfig {
            shell: "/bin/echo".to_string(),
            args: vec!["hello".to_string()],
            working_dir: std::env::current_dir().unwrap(),
            env: Vec::new(),
            size: PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            },
        };

        let id = pool.spawn(config, "test".to_string()).await;
        assert!(id.is_ok());

        let id = id.unwrap();
        assert_eq!(pool.session_count().await, 1);
        assert_eq!(pool.focused_id().await, Some(id));
    }

    #[tokio::test]
    async fn test_pool_capacity() {
        let config = PtyPoolConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let pool = PtyPool::new(config);

        // Spawn first session
        let _ = pool.spawn_default("test1".to_string()).await.unwrap();
        assert!(!pool.is_full().await);

        // Spawn second session
        let _ = pool.spawn_default("test2".to_string()).await.unwrap();
        assert!(pool.is_full().await);

        // Third spawn should fail
        let result = pool.spawn_default("test3".to_string()).await;
        assert!(matches!(result, Err(PtyError::PoolExhausted { .. })));
    }

    #[tokio::test]
    async fn test_focus_navigation() {
        let pool = PtyPool::with_defaults();

        let id1 = pool.spawn_default("test1".to_string()).await.unwrap();
        let id2 = pool.spawn_default("test2".to_string()).await.unwrap();
        let id3 = pool.spawn_default("test3".to_string()).await.unwrap();

        // First session should be focused
        assert_eq!(pool.focused_id().await, Some(id1));

        // Navigate forward
        let next = pool.focus_next().await;
        assert_eq!(next, Some(id2));
        assert_eq!(pool.focused_id().await, Some(id2));

        let next = pool.focus_next().await;
        assert_eq!(next, Some(id3));

        // Wrap around
        let next = pool.focus_next().await;
        assert_eq!(next, Some(id1));

        // Navigate backward
        let prev = pool.focus_prev().await;
        assert_eq!(prev, Some(id3));
    }
}
