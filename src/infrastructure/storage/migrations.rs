//! Database Migrations
//!
//! Schema creation and version management.

use super::sqlite::{Database, DatabaseError, DatabaseResult};
use rusqlite::Connection;

/// Current schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Run all migrations
pub fn run_migrations(db: &Database) -> DatabaseResult<()> {
    db.with_connection_mut(|conn| {
        // Create migrations table if not exists
        create_migrations_table(conn)?;

        // Get current version
        let current_version = get_schema_version(conn)?;

        // Apply migrations
        if current_version < 1 {
            migrate_v1(conn)?;
            set_schema_version(conn, 1)?;
        }

        Ok(())
    })
}

/// Create migrations tracking table
fn create_migrations_table(conn: &Connection) -> DatabaseResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        ",
    )?;
    Ok(())
}

/// Get current schema version
fn get_schema_version(conn: &Connection) -> DatabaseResult<i32> {
    let result: rusqlite::Result<i32> = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(version) => Ok(version),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Set schema version
fn set_schema_version(conn: &Connection, version: i32) -> DatabaseResult<()> {
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES (?)",
        [version],
    )?;
    Ok(())
}

/// Migration v1: Initial schema
fn migrate_v1(conn: &Connection) -> DatabaseResult<()> {
    conn.execute_batch(
        r"
        -- Sessions table
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            working_dir TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            metadata TEXT DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
        CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
        CREATE INDEX IF NOT EXISTS idx_sessions_working_dir ON sessions(working_dir);

        -- Session archives table
        CREATE TABLE IF NOT EXISTS session_archives (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            working_dir TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT NOT NULL,
            summary TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            metrics TEXT NOT NULL DEFAULT '{}',
            compression_level TEXT NOT NULL DEFAULT 'raw',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_archives_session_id ON session_archives(session_id);
        CREATE INDEX IF NOT EXISTS idx_archives_period ON session_archives(period_start, period_end);
        CREATE INDEX IF NOT EXISTS idx_archives_compression ON session_archives(compression_level);

        -- Full-text search for archives
        CREATE VIRTUAL TABLE IF NOT EXISTS archives_fts USING fts5(
            summary,
            tags,
            content='session_archives',
            content_rowid='rowid'
        );

        -- Triggers to keep FTS in sync
        CREATE TRIGGER IF NOT EXISTS archives_ai AFTER INSERT ON session_archives BEGIN
            INSERT INTO archives_fts(rowid, summary, tags)
            VALUES (NEW.rowid, NEW.summary, NEW.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS archives_ad AFTER DELETE ON session_archives BEGIN
            INSERT INTO archives_fts(archives_fts, rowid, summary, tags)
            VALUES ('delete', OLD.rowid, OLD.summary, OLD.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS archives_au AFTER UPDATE ON session_archives BEGIN
            INSERT INTO archives_fts(archives_fts, rowid, summary, tags)
            VALUES ('delete', OLD.rowid, OLD.summary, OLD.tags);
            INSERT INTO archives_fts(rowid, summary, tags)
            VALUES (NEW.rowid, NEW.summary, NEW.tags);
        END;

        -- Tasks table (for task graph persistence)
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            command TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            exit_code INTEGER,
            output_summary TEXT,
            full_log_path TEXT,
            started_at TEXT,
            completed_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_session_id ON tasks(session_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

        -- Task dependencies table
        CREATE TABLE IF NOT EXISTS task_dependencies (
            task_id TEXT NOT NULL,
            depends_on TEXT NOT NULL,
            PRIMARY KEY (task_id, depends_on),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (depends_on) REFERENCES tasks(id) ON DELETE CASCADE
        );

        -- Memory blocks table
        CREATE TABLE IF NOT EXISTS memory_blocks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            label TEXT NOT NULL,
            value TEXT NOT NULL,
            token_count INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
            UNIQUE (session_id, label)
        );

        CREATE INDEX IF NOT EXISTS idx_memory_session_label ON memory_blocks(session_id, label);

        -- Compacted outputs table
        CREATE TABLE IF NOT EXISTS compacted_outputs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            last_lines TEXT NOT NULL DEFAULT '[]',
            full_log_path TEXT NOT NULL,
            original_size INTEGER NOT NULL,
            compacted_size INTEGER NOT NULL,
            compacted_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_compacted_task_id ON compacted_outputs(task_id);

        -- MCP connections table
        CREATE TABLE IF NOT EXISTS mcp_connections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            server_name TEXT NOT NULL,
            connected INTEGER NOT NULL DEFAULT 0,
            tools_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
            UNIQUE (session_id, server_name)
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_session ON mcp_connections(session_id);
        ",
    )?;

    Ok(())
}

/// Check if migrations are up to date
pub fn check_migrations(db: &Database) -> DatabaseResult<bool> {
    db.with_connection(|conn| {
        let version = get_schema_version(conn)?;
        Ok(version >= SCHEMA_VERSION)
    })
}

/// Get applied migrations
pub fn get_applied_migrations(db: &Database) -> DatabaseResult<Vec<(i32, String)>> {
    db.with_connection(|conn| {
        let mut stmt =
            conn.prepare("SELECT version, applied_at FROM schema_migrations ORDER BY version")?;

        let migrations = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(migrations)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_migrations() {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();

        // Verify tables exist
        let count = db.table_count("sessions").unwrap();
        assert_eq!(count, 0);

        let count = db.table_count("session_archives").unwrap();
        assert_eq!(count, 0);

        let count = db.table_count("tasks").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_migrations_idempotent() {
        let db = Database::open_in_memory().unwrap();

        // Run twice
        run_migrations(&db).unwrap();
        run_migrations(&db).unwrap();

        // Should still work
        assert!(check_migrations(&db).unwrap());
    }

    #[test]
    fn test_get_applied_migrations() {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();

        let migrations = get_applied_migrations(&db).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].0, 1);
    }
}
