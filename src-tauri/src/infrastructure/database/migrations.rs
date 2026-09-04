use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::errors::AppError;

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
    Migration {
        version: 3,
        sql: include_str!("../../../migrations/003_add_due_date.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("../../../migrations/004_add_tags.sql"),
    },
    Migration {
        version: 5,
        sql: include_str!("../../../migrations/005_add_events.sql"),
    },
    Migration {
        version: 6,
        sql: include_str!("../../../migrations/006_float_modes.sql"),
    },
    Migration {
        version: 7,
        sql: include_str!("../../../migrations/007_companion.sql"),
    },
    Migration {
        version: 8,
        sql: include_str!("../../../migrations/008_companion_fill.sql"),
    },
    Migration {
        version: 9,
        sql: include_str!("../../../migrations/009_normalize_tags.sql"),
    },
    Migration {
        version: 10,
        sql: include_str!("../../../migrations/010_ui_theme_locale.sql"),
    },
    Migration {
        version: 11,
        sql: include_str!("../../../migrations/011_list_default_view.sql"),
    },
    Migration {
        version: 12,
        sql: include_str!("../../../migrations/012_default_view_all.sql"),
    },
    Migration {
        version: 13,
        sql: include_str!("../../../migrations/013_companion_visible_count_10.sql"),
    },
];

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

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let current = current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        // SQL + schema_version 写入同一事务，避免 ALTER 成功后崩溃导致无法重跑
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| AppError::DbError {
                message: format!("migration v{} begin failed: {error}", migration.version),
            })?;

        tx.execute_batch(migration.sql)
            .map_err(|error| AppError::DbError {
                message: format!("migration v{} failed: {error}", migration.version),
            })?;

        let applied_at = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![migration.version, applied_at],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;

        tx.commit().map_err(|error| AppError::DbError {
            message: format!("migration v{} commit failed: {error}", migration.version),
        })?;
    }

    Ok(())
}
