use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::errors::AppError;
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
}
