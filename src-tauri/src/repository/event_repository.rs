use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::errors::AppError;
use crate::events::EventDto;
use crate::infrastructure::database::Database;

pub struct EventRepository;

impl EventRepository {
    pub fn write(
        db: &Database,
        entity_type: &str,
        entity_id: &str,
        event_type: &str,
        payload: &str,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO events (id, entity_type, entity_id, event_type, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, entity_type, entity_id, event_type, payload, now],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(())
        })
    }

    /// 按 created_at 升序返回 after 之后的事件（不含 after 本身）。
    /// 无 after 时先按最新取出再反转为时间正序，与有游标路径同为升序。
    pub fn list_since(
        db: &Database,
        after: Option<&str>,
        limit: u32,
    ) -> Result<Vec<EventDto>, AppError> {
        db.with_conn(|conn| {
            let limit_i = i64::from(limit);
            let mut stmt = if after.is_some() {
                conn.prepare(
                    "SELECT id, entity_type, entity_id, event_type, payload, created_at
                     FROM events
                     WHERE created_at > ?1
                     ORDER BY created_at ASC, id ASC
                     LIMIT ?2",
                )
            } else {
                conn.prepare(
                    "SELECT id, entity_type, entity_id, event_type, payload, created_at
                     FROM events
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?1",
                )
            }
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

            let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<EventDto> {
                Ok(EventDto {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    event_type: row.get(3)?,
                    payload: row.get(4)?,
                    created_at: row.get(5)?,
                })
            };

            let rows = if let Some(cursor) = after {
                stmt.query_map(params![cursor, limit_i], map_row)
            } else {
                stmt.query_map(params![limit_i], map_row)
            }
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?);
            }
            if after.is_none() {
                items.reverse();
            }
            Ok(items)
        })
    }
}
