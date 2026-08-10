use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use tauri::Manager;

use crate::errors::AppError;
use crate::infrastructure::filesystem;

pub struct Database {
    conn: Mutex<Connection>,
}

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("../../../migrations/001_init.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../../../migrations/002_seed_settings.sql"),
    },
];

impl Database {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, AppError> {
        filesystem::ensure_dirs(app)?;
        let db_path = filesystem::db_path(app)?;
        let conn = Connection::open(db_path).map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn with_conn<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.conn.lock().map_err(|_| AppError::InternalError {
            message: "database lock poisoned".into(),
        })?;
        f(&conn)
    }
}

fn current_version(conn: &Connection) -> Result<i32, AppError> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?
        .is_some();

    if !exists {
        return Ok(0);
    }

    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .map_err(|error| AppError::DbError {
        message: error.to_string(),
    })
}

fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let current = current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        conn.execute_batch(migration.sql)
            .map_err(|error| AppError::DbError {
                message: format!(
                    "migration v{} failed: {error}",
                    migration.version
                ),
            })?;

        let applied_at = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![migration.version, applied_at],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
    }

    Ok(())
}
