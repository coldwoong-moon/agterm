//! Storage Backend
//!
//! `SQLite` implementation, migrations, and archive management.

pub mod archive;
pub mod migrations;
pub mod sqlite;

pub use archive::{ArchiveQuery, ArchiveRepository, ArchiveSearchResult};
pub use migrations::run_migrations;
pub use sqlite::{Database, DatabaseConfig};
