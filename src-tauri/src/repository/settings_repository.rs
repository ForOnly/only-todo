use rusqlite::{params, OptionalExtension};

use crate::domain::{SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::infrastructure::database::Database;

pub struct SettingsRepository;

impl SettingsRepository {
    pub fn get(db: &Database) -> Result<SettingsDto, AppError> {
        let notification_enabled = Self::get_value(db, "notification.enabled")?
            .map(|value| value == "true")
            .unwrap_or(true);
        let list_default_sort = Self::get_value(db, "list.default_sort")?
            .unwrap_or_else(|| r#"{"sort_by":"priority","sort_order":"desc"}"#.into());

        Ok(SettingsDto {
            notification_enabled,
            list_default_sort,
        })
    }

    pub fn update(db: &Database, dto: &UpdateSettingsDto) -> Result<SettingsDto, AppError> {
        if let Some(enabled) = dto.notification_enabled {
            Self::set_value(db, "notification.enabled", if enabled { "true" } else { "false" })?;
        }
        if let Some(sort) = &dto.list_default_sort {
            Self::set_value(db, "list.default_sort", sort)?;
        }
        Self::get(db)
    }

    pub fn get_value(db: &Database, key: &str) -> Result<Option<String>, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    fn set_value(db: &Database, key: &str, value: &str) -> Result<(), AppError> {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(())
        })
    }
}
