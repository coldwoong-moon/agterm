//! Archive Repository
//!
//! CRUD operations for session archives with full-text search.

use super::sqlite::{Database, DatabaseError, DatabaseResult};
use crate::domain::session::{CompressionLevel, SessionArchive, SessionMetrics};
use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Archive search query
#[derive(Debug, Clone, Default)]
pub struct ArchiveQuery {
    /// Full-text search query
    pub search: Option<String>,
    /// Filter by working directory (prefix match)
    pub working_dir: Option<PathBuf>,
    /// Filter by date range
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by compression level
    pub compression_level: Option<CompressionLevel>,
    /// Filter by tags (any match)
    pub tags: Vec<String>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl ArchiveQuery {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    #[must_use]
    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.date_range = Some((start, end));
        self
    }

    #[must_use]
    pub fn compression_level(mut self, level: CompressionLevel) -> Self {
        self.compression_level = Some(level);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Archive search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSearchResult {
    /// Archive data
    pub archive: SessionArchive,
    /// Search relevance score (if using FTS)
    pub score: Option<f64>,
    /// Highlighted snippet (if using FTS)
    pub snippet: Option<String>,
}

/// Archive repository
pub struct ArchiveRepository {
    db: Database,
}

impl ArchiveRepository {
    /// Create new repository
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Insert a new archive
    pub fn insert(&self, archive: &SessionArchive) -> DatabaseResult<()> {
        self.db.with_connection(|conn| {
            let tags_json = serde_json::to_string(&archive.tags).unwrap_or_default();
            let metrics_json = serde_json::to_string(&archive.metrics).unwrap_or_default();
            let compression = format!("{:?}", archive.compression_level).to_lowercase();

            conn.execute(
                r"
                INSERT INTO session_archives
                (id, session_id, working_dir, period_start, period_end, summary, tags, metrics, compression_level, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                params![
                    archive.id.to_string(),
                    archive.session_id.to_string(),
                    archive.working_dir.to_string_lossy(),
                    archive.period.0.to_rfc3339(),
                    archive.period.1.to_rfc3339(),
                    archive.summary,
                    tags_json,
                    metrics_json,
                    compression,
                    archive.created_at.to_rfc3339(),
                ],
            )?;

            Ok(())
        })
    }

    /// Get archive by ID
    pub fn get(&self, id: &Uuid) -> DatabaseResult<Option<SessionArchive>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT * FROM session_archives WHERE id = ?")?;

            let result = stmt.query_row([id.to_string()], Self::row_to_archive);

            match result {
                Ok(archive) => Ok(Some(archive)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DatabaseError::Sqlite(e)),
            }
        })
    }

    /// Get archive by session ID
    pub fn get_by_session(&self, session_id: &Uuid) -> DatabaseResult<Option<SessionArchive>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM session_archives WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
            )?;

            let result = stmt.query_row([session_id.to_string()], |row| {
                Self::row_to_archive(row)
            });

            match result {
                Ok(archive) => Ok(Some(archive)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DatabaseError::Sqlite(e)),
            }
        })
    }

    /// Search archives
    pub fn search(&self, query: &ArchiveQuery) -> DatabaseResult<Vec<ArchiveSearchResult>> {
        self.db.with_connection(|conn| {
            // Build query based on whether we're using FTS
            if let Some(ref search_text) = query.search {
                self.search_fts(conn, search_text, query)
            } else {
                self.search_standard(conn, query)
            }
        })
    }

    /// Full-text search
    fn search_fts(
        &self,
        conn: &rusqlite::Connection,
        search_text: &str,
        query: &ArchiveQuery,
    ) -> DatabaseResult<Vec<ArchiveSearchResult>> {
        let mut sql = String::from(
            r"
            SELECT sa.*,
                   bm25(archives_fts) as score,
                   snippet(archives_fts, 0, '<b>', '</b>', '...', 32) as snippet
            FROM session_archives sa
            JOIN archives_fts ON sa.rowid = archives_fts.rowid
            WHERE archives_fts MATCH ?
            ",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(search_text.to_string())];

        // Add filters
        self.add_filters(&mut sql, &mut params, query);

        sql.push_str(" ORDER BY score");

        // Add limit/offset
        self.add_pagination(&mut sql, &mut params, query);

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(std::convert::AsRef::as_ref).collect();

        let results = stmt
            .query_map(param_refs.as_slice(), |row| {
                let archive = Self::row_to_archive(row)?;
                let score: f64 = row.get("score")?;
                let snippet: String = row.get("snippet")?;

                Ok(ArchiveSearchResult {
                    archive,
                    score: Some(score),
                    snippet: Some(snippet),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Standard search (no FTS)
    fn search_standard(
        &self,
        conn: &rusqlite::Connection,
        query: &ArchiveQuery,
    ) -> DatabaseResult<Vec<ArchiveSearchResult>> {
        let mut sql = String::from("SELECT * FROM session_archives WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Add filters
        self.add_filters(&mut sql, &mut params, query);

        sql.push_str(" ORDER BY created_at DESC");

        // Add limit/offset
        self.add_pagination(&mut sql, &mut params, query);

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(std::convert::AsRef::as_ref).collect();

        let results = stmt
            .query_map(param_refs.as_slice(), |row| {
                let archive = Self::row_to_archive(row)?;
                Ok(ArchiveSearchResult {
                    archive,
                    score: None,
                    snippet: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Add filter clauses to SQL
    fn add_filters(
        &self,
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &ArchiveQuery,
    ) {
        if let Some(ref dir) = query.working_dir {
            sql.push_str(" AND working_dir LIKE ?");
            params.push(Box::new(format!("{}%", dir.to_string_lossy())));
        }

        if let Some((start, end)) = &query.date_range {
            sql.push_str(" AND period_start >= ? AND period_end <= ?");
            params.push(Box::new(start.to_rfc3339()));
            params.push(Box::new(end.to_rfc3339()));
        }

        if let Some(ref level) = query.compression_level {
            sql.push_str(" AND compression_level = ?");
            params.push(Box::new(format!("{level:?}").to_lowercase()));
        }

        // Tag filtering (JSON contains)
        for tag in &query.tags {
            sql.push_str(" AND tags LIKE ?");
            params.push(Box::new(format!("%\"{tag}\"")));
        }
    }

    /// Add pagination clauses
    fn add_pagination(
        &self,
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &ArchiveQuery,
    ) {
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(Box::new(limit as i64));
        }

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset as i64));
        }
    }

    /// Convert row to `SessionArchive`
    fn row_to_archive(row: &Row) -> rusqlite::Result<SessionArchive> {
        let id_str: String = row.get("id")?;
        let session_id_str: String = row.get("session_id")?;
        let working_dir_str: String = row.get("working_dir")?;
        let period_start_str: String = row.get("period_start")?;
        let period_end_str: String = row.get("period_end")?;
        let summary: String = row.get("summary")?;
        let tags_json: String = row.get("tags")?;
        let metrics_json: String = row.get("metrics")?;
        let compression_str: String = row.get("compression_level")?;
        let created_at_str: String = row.get("created_at")?;

        let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
        let session_id = Uuid::parse_str(&session_id_str).unwrap_or_else(|_| Uuid::new_v4());
        let working_dir = PathBuf::from(working_dir_str);
        let period_start = DateTime::parse_from_rfc3339(&period_start_str)
            .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc));
        let period_end = DateTime::parse_from_rfc3339(&period_end_str)
            .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc));
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let metrics: SessionMetrics = serde_json::from_str(&metrics_json).unwrap_or_default();
        let compression_level = match compression_str.as_str() {
            "compacted" => CompressionLevel::Compacted,
            "summarized" => CompressionLevel::Summarized,
            "rolled" => CompressionLevel::Rolled,
            _ => CompressionLevel::Raw,
        };
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc));

        Ok(SessionArchive {
            id,
            session_id,
            working_dir,
            period: (period_start, period_end),
            summary,
            tags,
            metrics,
            compression_level,
            created_at,
        })
    }

    /// Update archive
    pub fn update(&self, archive: &SessionArchive) -> DatabaseResult<()> {
        self.db.with_connection(|conn| {
            let tags_json = serde_json::to_string(&archive.tags).unwrap_or_default();
            let metrics_json = serde_json::to_string(&archive.metrics).unwrap_or_default();
            let compression = format!("{:?}", archive.compression_level).to_lowercase();

            conn.execute(
                r"
                UPDATE session_archives
                SET summary = ?, tags = ?, metrics = ?, compression_level = ?
                WHERE id = ?
                ",
                params![
                    archive.summary,
                    tags_json,
                    metrics_json,
                    compression,
                    archive.id.to_string(),
                ],
            )?;

            Ok(())
        })
    }

    /// Delete archive
    pub fn delete(&self, id: &Uuid) -> DatabaseResult<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "DELETE FROM session_archives WHERE id = ?",
                [id.to_string()],
            )?;
            Ok(())
        })
    }

    /// Get recent archives
    pub fn recent(&self, limit: usize) -> DatabaseResult<Vec<SessionArchive>> {
        self.db.with_connection(|conn| {
            let mut stmt =
                conn.prepare("SELECT * FROM session_archives ORDER BY created_at DESC LIMIT ?")?;

            let archives = stmt
                .query_map([limit as i64], Self::row_to_archive)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(archives)
        })
    }

    /// Get archives by working directory
    pub fn by_working_dir(&self, dir: &PathBuf) -> DatabaseResult<Vec<SessionArchive>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT * FROM session_archives WHERE working_dir = ? ORDER BY created_at DESC",
            )?;

            let archives = stmt
                .query_map([dir.to_string_lossy()], Self::row_to_archive)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(archives)
        })
    }

    /// Count total archives
    pub fn count(&self) -> DatabaseResult<i64> {
        self.db.table_count("session_archives")
    }

    /// Get all unique tags
    pub fn all_tags(&self) -> DatabaseResult<Vec<String>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT DISTINCT tags FROM session_archives")?;

            let mut all_tags: Vec<String> = Vec::new();

            let rows = stmt.query_map([], |row| {
                let tags_json: String = row.get(0)?;
                Ok(tags_json)
            })?;

            for row in rows {
                if let Ok(tags_json) = row {
                    if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tags_json) {
                        for tag in tags {
                            if !all_tags.contains(&tag) {
                                all_tags.push(tag);
                            }
                        }
                    }
                }
            }

            all_tags.sort();
            Ok(all_tags)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::Session;
    use crate::infrastructure::storage::migrations::run_migrations;
    use rusqlite::params;

    fn setup_test_db() -> (Database, ArchiveRepository) {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();
        let repo = ArchiveRepository::new(db.clone());
        (db, repo)
    }

    /// Helper to insert a session into the database
    fn insert_session(db: &Database, session: &Session) {
        db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, working_dir, started_at, status) VALUES (?, ?, ?, ?)",
                params![
                    session.id.to_string(),
                    session.working_dir.to_string_lossy(),
                    session.started_at.to_rfc3339(),
                    "active"
                ],
            )?;
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_insert_and_get() {
        let (db, repo) = setup_test_db();

        let session = Session::new(PathBuf::from("/tmp/test"));
        insert_session(&db, &session);

        let archive = SessionArchive::from_session(
            &session,
            "Test summary".to_string(),
            vec!["test".to_string()],
        );

        repo.insert(&archive).unwrap();

        let retrieved = repo.get(&archive.id).unwrap().unwrap();
        assert_eq!(retrieved.id, archive.id);
        assert_eq!(retrieved.summary, "Test summary");
    }

    #[test]
    fn test_search_standard() {
        let (db, repo) = setup_test_db();

        let session = Session::new(PathBuf::from("/tmp/test"));
        insert_session(&db, &session);

        let archive = SessionArchive::from_session(
            &session,
            "Test summary".to_string(),
            vec!["rust".to_string()],
        );

        repo.insert(&archive).unwrap();

        let query = ArchiveQuery::new().working_dir("/tmp");
        let results = repo.search(&query).unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_recent() {
        let (db, repo) = setup_test_db();

        for i in 0..5 {
            let session = Session::new(PathBuf::from(format!("/tmp/test{i}")));
            insert_session(&db, &session);

            let archive = SessionArchive::from_session(&session, format!("Summary {i}"), vec![]);
            repo.insert(&archive).unwrap();
        }

        let recent = repo.recent(3).unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_update() {
        let (db, repo) = setup_test_db();

        let session = Session::new(PathBuf::from("/tmp/test"));
        insert_session(&db, &session);

        let mut archive = SessionArchive::from_session(&session, "Original".to_string(), vec![]);

        repo.insert(&archive).unwrap();

        archive.summary = "Updated".to_string();
        repo.update(&archive).unwrap();

        let retrieved = repo.get(&archive.id).unwrap().unwrap();
        assert_eq!(retrieved.summary, "Updated");
    }

    #[test]
    fn test_delete() {
        let (db, repo) = setup_test_db();

        let session = Session::new(PathBuf::from("/tmp/test"));
        insert_session(&db, &session);

        let archive = SessionArchive::from_session(&session, "Test".to_string(), vec![]);

        repo.insert(&archive).unwrap();
        assert!(repo.get(&archive.id).unwrap().is_some());

        repo.delete(&archive.id).unwrap();
        assert!(repo.get(&archive.id).unwrap().is_none());
    }

    #[test]
    fn test_all_tags() {
        let (db, repo) = setup_test_db();

        let session1 = Session::new(PathBuf::from("/tmp/test1"));
        insert_session(&db, &session1);
        let archive1 = SessionArchive::from_session(
            &session1,
            "Summary 1".to_string(),
            vec!["rust".to_string(), "async".to_string()],
        );

        let session2 = Session::new(PathBuf::from("/tmp/test2"));
        insert_session(&db, &session2);
        let archive2 = SessionArchive::from_session(
            &session2,
            "Summary 2".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
        );

        repo.insert(&archive1).unwrap();
        repo.insert(&archive2).unwrap();

        let tags = repo.all_tags().unwrap();
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"async".to_string()));
        assert!(tags.contains(&"cli".to_string()));
    }
}
