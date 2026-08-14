use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::reminder::{
    CreateReminderDto, DueReminder, Reminder, RepeatType, UpdateReminderDto,
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;

pub struct ReminderRepository;

impl ReminderRepository {
    pub fn count_by_todo(db: &Database, todo_id: &str) -> Result<i64, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM reminders WHERE todo_id = ?1 AND enabled = 1",
                params![todo_id],
                |row| row.get(0),
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    /// 软删任务时禁用其提醒，避免周期项继续被 scheduler 推进。
    pub(crate) fn disable_by_todo_on(conn: &Connection, todo_id: &str) -> Result<(), AppError> {
        let updated_at = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE reminders SET enabled = 0, updated_at = ?1 WHERE todo_id = ?2 AND enabled = 1",
            params![updated_at, todo_id],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        Ok(())
    }

    /// 回收站恢复后重开提醒：未来的一次性/周期直接启用；已过期周期跳到下一槽；已过期一次性保持关闭。
    pub(crate) fn reenable_after_restore_on(
        conn: &Connection,
        todo_id: &str,
    ) -> Result<(), AppError> {
        let reminders = Self::list_by_todo_on(conn, todo_id)?;
        let now = Utc::now();
        for reminder in reminders {
            if reminder.enabled {
                continue;
            }
            match reminder.repeat_type {
                RepeatType::None => {
                    if reminder.next_trigger_at > now {
                        Self::set_enabled_on(conn, &reminder.id, true)?;
                    }
                }
                RepeatType::Daily | RepeatType::Weekly => {
                    if reminder.next_trigger_at > now {
                        Self::set_enabled_on(conn, &reminder.id, true)?;
                    } else {
                        // 在 trash 期间错过的周期槽：推进到未来并启用（不写 triggered 审计）
                        Self::advance_on(conn, &reminder)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn set_enabled_on(conn: &Connection, id: &str, enabled: bool) -> Result<(), AppError> {
        let updated_at = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE reminders SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i32, updated_at, id],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        Ok(())
    }

    pub fn create(
        db: &Database,
        dto: &CreateReminderDto,
        repeat_type: RepeatType,
    ) -> Result<Reminder, AppError> {
        let remind_at = parse_datetime(&dto.remind_at)?;
        let now = Utc::now();
        let reminder = Reminder {
            id: Uuid::new_v4().to_string(),
            todo_id: dto.todo_id.clone(),
            remind_at,
            repeat_type,
            repeat_config: "{}".into(),
            snooze_count: 0,
            next_trigger_at: remind_at,
            enabled: true,
            created_at: now,
            updated_at: now,
        };

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO reminders (id, todo_id, remind_at, repeat_type, repeat_config, snooze_count, next_trigger_at, enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    reminder.id,
                    reminder.todo_id,
                    reminder.remind_at.to_rfc3339(),
                    reminder.repeat_type.as_str(),
                    reminder.repeat_config,
                    reminder.snooze_count,
                    reminder.next_trigger_at.to_rfc3339(),
                    reminder.enabled as i32,
                    reminder.created_at.to_rfc3339(),
                    reminder.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(reminder)
        })
    }

    pub fn update(db: &Database, dto: &UpdateReminderDto) -> Result<Reminder, AppError> {
        let mut reminder = Self::get_by_id(db, &dto.id)?;

        if let Some(remind_at) = &dto.remind_at {
            let parsed = parse_datetime(remind_at)?;
            reminder.remind_at = parsed;
            reminder.next_trigger_at = parsed;
        }
        if let Some(enabled) = dto.enabled {
            reminder.enabled = enabled;
        }
        reminder.updated_at = Utc::now();

        db.with_conn(|conn| {
            conn.execute(
                "UPDATE reminders SET remind_at = ?1, next_trigger_at = ?2, enabled = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    reminder.remind_at.to_rfc3339(),
                    reminder.next_trigger_at.to_rfc3339(),
                    reminder.enabled as i32,
                    reminder.updated_at.to_rfc3339(),
                    reminder.id,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(reminder)
        })
    }

    pub fn delete(db: &Database, id: &str) -> Result<(), AppError> {
        let rows = db.with_conn(|conn| {
            conn.execute("DELETE FROM reminders WHERE id = ?1", params![id])
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })
        })?;

        if rows == 0 {
            return Err(AppError::NotFound {
                message: format!("reminder {id} not found"),
            });
        }
        Ok(())
    }

    pub fn get_by_id(db: &Database, id: &str) -> Result<Reminder, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, todo_id, remind_at, repeat_type, repeat_config, snooze_count, next_trigger_at, enabled, created_at, updated_at
                 FROM reminders WHERE id = ?1",
                params![id],
                map_row,
            )
            .optional()
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?
            .ok_or_else(|| AppError::NotFound {
                message: format!("reminder {id} not found"),
            })
        })
    }

    pub fn list_by_todo(db: &Database, todo_id: &str) -> Result<Vec<Reminder>, AppError> {
        db.with_conn(|conn| Self::list_by_todo_on(conn, todo_id))
    }

    fn list_by_todo_on(conn: &Connection, todo_id: &str) -> Result<Vec<Reminder>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, todo_id, remind_at, repeat_type, repeat_config, snooze_count, next_trigger_at, enabled, created_at, updated_at
                 FROM reminders WHERE todo_id = ?1 ORDER BY next_trigger_at ASC",
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

        let items = stmt
            .query_map(params![todo_id], map_row)
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

        Ok(items)
    }

    pub fn find_due(db: &Database, now: &DateTime<Utc>) -> Result<Vec<DueReminder>, AppError> {
        db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT r.id, r.todo_id, r.remind_at, r.repeat_type, r.repeat_config, r.snooze_count, r.next_trigger_at, r.enabled, r.created_at, r.updated_at,
                            t.title, t.deleted_at
                     FROM reminders r
                     INNER JOIN todos t ON t.id = r.todo_id
                     WHERE r.enabled = 1 AND r.next_trigger_at <= ?1
                     ORDER BY r.next_trigger_at ASC",
                )
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;

            let items = stmt
                .query_map(params![now.to_rfc3339()], |row| {
                    let reminder = map_row(row)?;
                    let todo_title: String = row.get(10)?;
                    let deleted_at: Option<String> = row.get(11)?;
                    Ok(DueReminder {
                        reminder,
                        todo_title,
                        todo_deleted: deleted_at.is_some(),
                    })
                })
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;

            Ok(items)
        })
    }

    /// 推进下次触发时间：一次性禁用；周期则跳到严格大于 now 的下一槽，避免 downtime 后每 tick 刷屏。
    pub fn advance(db: &Database, reminder: &Reminder) -> Result<(), AppError> {
        db.with_conn(|conn| Self::advance_on(conn, reminder))
    }

    fn advance_on(conn: &Connection, reminder: &Reminder) -> Result<(), AppError> {
        let now = Utc::now();
        let updated_at = now.to_rfc3339();
        let (enabled, next_trigger_at) = match reminder.repeat_type {
            RepeatType::None => (0, reminder.next_trigger_at.to_rfc3339()),
            RepeatType::Daily => {
                let mut next = reminder.next_trigger_at + ChronoDuration::days(1);
                while next <= now {
                    next += ChronoDuration::days(1);
                }
                (1, next.to_rfc3339())
            }
            RepeatType::Weekly => {
                let mut next = reminder.next_trigger_at + ChronoDuration::weeks(1);
                while next <= now {
                    next += ChronoDuration::weeks(1);
                }
                (1, next.to_rfc3339())
            }
        };

        conn.execute(
            "UPDATE reminders SET enabled = ?1, next_trigger_at = ?2, updated_at = ?3 WHERE id = ?4",
            params![enabled, next_trigger_at, updated_at, reminder.id],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        Ok(())
    }

    pub fn snooze(db: &Database, id: &str, minutes: i64) -> Result<Reminder, AppError> {
        let mut reminder = Self::get_by_id(db, id)?;
        reminder.snooze_count += 1;
        reminder.next_trigger_at = Utc::now() + ChronoDuration::minutes(minutes);
        reminder.enabled = true;
        reminder.updated_at = Utc::now();

        db.with_conn(|conn| {
            conn.execute(
                "UPDATE reminders SET snooze_count = ?1, next_trigger_at = ?2, enabled = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    reminder.snooze_count,
                    reminder.next_trigger_at.to_rfc3339(),
                    reminder.enabled as i32,
                    reminder.updated_at.to_rfc3339(),
                    reminder.id,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(reminder)
        })
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Reminder> {
    let repeat_type_str: String = row.get(3)?;
    let remind_at: String = row.get(2)?;
    let next_trigger_at: String = row.get(6)?;
    let created_at: String = row.get(8)?;
    let updated_at: String = row.get(9)?;
    let enabled: i32 = row.get(7)?;

    Ok(Reminder {
        id: row.get(0)?,
        todo_id: row.get(1)?,
        remind_at: parse_datetime_unchecked(&remind_at),
        repeat_type: RepeatType::from_str(&repeat_type_str).unwrap_or(RepeatType::None),
        repeat_config: row.get(4)?,
        snooze_count: row.get(5)?,
        next_trigger_at: parse_datetime_unchecked(&next_trigger_at),
        enabled: enabled != 0,
        created_at: parse_datetime_unchecked(&created_at),
        updated_at: parse_datetime_unchecked(&updated_at),
    })
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AppError::ValidationError {
            message: format!("invalid datetime: {error}"),
        })
}

fn parse_datetime_unchecked(value: &str) -> DateTime<Utc> {
    parse_datetime(value).unwrap_or_else(|_| Utc::now())
}
