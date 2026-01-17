//! SQLite Database
//!
//! Core database connection and configuration.

use rusqlite::{Connection, OpenFlags, Result as SqliteResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Database errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Migration failed: {0}")]
    Migration(String),
}

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database file path
    pub path: PathBuf,
    /// Enable WAL mode for better concurrency
    pub wal_mode: bool,
    /// Enable foreign keys
    pub foreign_keys: bool,
    /// Journal size limit (bytes)
    pub journal_size_limit: Option<i64>,
    /// Cache size (pages, negative = KB)
    pub cache_size: Option<i32>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".agterm/agterm.db"),
            wal_mode: true,
            foreign_keys: true,
            journal_size_limit: Some(32 * 1024 * 1024), // 32MB
            cache_size: Some(-64000),                   // 64MB
        }
    }
}

impl DatabaseConfig {
    /// Create config for in-memory database
    pub fn in_memory() -> Self {
        Self {
            path: PathBuf::from(":memory:"),
            wal_mode: false, // WAL not supported for in-memory
            foreign_keys: true,
            journal_size_limit: None,
            cache_size: Some(-64000),
        }
    }

    /// Create config with custom path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

/// Thread-safe database wrapper
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create database
    pub fn open(config: DatabaseConfig) -> DatabaseResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;

        let conn = Connection::open_with_flags(&config.path, flags)?;

        // Apply configuration
        Self::configure_connection(&conn, &config)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        })
    }

    /// Open in-memory database
    pub fn open_in_memory() -> DatabaseResult<Self> {
        Self::open(DatabaseConfig::in_memory())
    }

    /// Configure connection pragmas
    fn configure_connection(conn: &Connection, config: &DatabaseConfig) -> DatabaseResult<()> {
        // Enable foreign keys
        if config.foreign_keys {
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        }

        // Enable WAL mode
        if config.wal_mode {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }

        // Set journal size limit
        if let Some(limit) = config.journal_size_limit {
            conn.execute_batch(&format!("PRAGMA journal_size_limit = {};", limit))?;
        }

        // Set cache size
        if let Some(size) = config.cache_size {
            conn.execute_batch(&format!("PRAGMA cache_size = {};", size))?;
        }

        // Performance optimizations
        conn.execute_batch(
            "
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
            ",
        )?;

        Ok(())
    }

    /// Get database path
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Get config
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Execute with connection
    pub fn with_connection<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&Connection) -> DatabaseResult<T>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DatabaseError::Lock(e.to_string()))?;
        f(&conn)
    }

    /// Execute with mutable connection (for transactions)
    pub fn with_connection_mut<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&mut Connection) -> DatabaseResult<T>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| DatabaseError::Lock(e.to_string()))?;
        f(&mut conn)
    }

    /// Execute a transaction
    pub fn transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> DatabaseResult<T>,
    {
        self.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            let result = f(&tx)?;
            tx.commit()?;
            Ok(result)
        })
    }

    /// Check if database exists and is valid
    pub fn is_valid(&self) -> bool {
        self.with_connection(|conn| {
            conn.execute_batch("SELECT 1;")
                .map_err(DatabaseError::from)
        })
        .is_ok()
    }

    /// Get database size in bytes
    pub fn size(&self) -> DatabaseResult<u64> {
        if self.config.path.as_os_str() == ":memory:" {
            return Ok(0);
        }
        Ok(std::fs::metadata(&self.config.path)?.len())
    }

    /// Vacuum database to reclaim space
    pub fn vacuum(&self) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            conn.execute_batch("VACUUM;")?;
            Ok(())
        })
    }

    /// Checkpoint WAL
    pub fn checkpoint(&self) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
            Ok(())
        })
    }

    /// Get table row count
    pub fn table_count(&self, table: &str) -> DatabaseResult<i64> {
        self.with_connection(|conn| {
            let count: i64 =
                conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                    row.get(0)
                })?;
            Ok(count)
        })
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory().unwrap();
        assert!(db.is_valid());
    }

    #[test]
    fn test_with_connection() {
        let db = Database::open_in_memory().unwrap();

        db.with_connection(|conn| {
            conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY)")?;
            Ok(())
        })
        .unwrap();

        let count = db.table_count("test").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_transaction() {
        let db = Database::open_in_memory().unwrap();

        db.with_connection(|conn| {
            conn.execute_batch("CREATE TABLE test (value TEXT)")?;
            Ok(())
        })
        .unwrap();

        db.transaction(|tx| {
            tx.execute("INSERT INTO test VALUES (?)", ["hello"])?;
            tx.execute("INSERT INTO test VALUES (?)", ["world"])?;
            Ok(())
        })
        .unwrap();

        let count = db.table_count("test").unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_clone_shares_connection() {
        let db1 = Database::open_in_memory().unwrap();
        let db2 = db1.clone();

        db1.with_connection(|conn| {
            conn.execute_batch("CREATE TABLE shared (id INTEGER)")?;
            Ok(())
        })
        .unwrap();

        // Should see the same table
        let count = db2.table_count("shared").unwrap();
        assert_eq!(count, 0);
    }
}
