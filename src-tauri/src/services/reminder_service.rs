use chrono::{DateTime, Duration as ChronoDuration, Utc};

use crate::domain::reminder::{
    CreateReminderDto, ReminderDto, RepeatType, UpdateReminderDto,
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::event_repository::EventRepository;
use crate::repository::reminder_repository::ReminderRepository;
use crate::repository::todo_repository::TodoRepository;

const MAX_REMINDERS_PER_TODO: i64 = 5;
const SNOOZE_OPTIONS: [i64; 4] = [5, 15, 30, 60];
const MAX_SNOOZE_COUNT: i32 = 3;

pub struct ReminderService;

impl ReminderService {
    pub fn create(db: &Database, dto: CreateReminderDto) -> Result<ReminderDto, AppError> {
        let todo = TodoRepository::get_by_id(db, &dto.todo_id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {} not found", dto.todo_id),
            });
        }

        let count = ReminderRepository::count_by_todo(db, &dto.todo_id)?;
        if count >= MAX_REMINDERS_PER_TODO {
            return Err(AppError::Conflict {
                message: format!("todo already has {MAX_REMINDERS_PER_TODO} reminders"),
            });
        }

        let remind_at = parse_future_datetime(&dto.remind_at)?;
        if remind_at <= Utc::now() {
            return Err(AppError::ValidationError {
                message: "remind_at must be in the future".into(),
            });
        }

        let repeat_type = dto.repeat_type.unwrap_or(RepeatType::None);
        let reminder = ReminderRepository::create(db, &dto, repeat_type)?;
        Ok(reminder.into())
    }

    pub fn update(db: &Database, dto: UpdateReminderDto) -> Result<ReminderDto, AppError> {
        if let Some(remind_at) = &dto.remind_at {
            let parsed = parse_future_datetime(remind_at)?;
            if parsed <= Utc::now() {
                return Err(AppError::ValidationError {
                    message: "remind_at must be in the future".into(),
                });
            }
        }

        let reminder = ReminderRepository::update(db, &dto)?;
        Ok(reminder.into())
    }

    pub fn delete(db: &Database, id: &str) -> Result<(), AppError> {
        ReminderRepository::delete(db, id)
    }

    pub fn list(db: &Database, todo_id: &str) -> Result<Vec<ReminderDto>, AppError> {
        let todo = TodoRepository::get_by_id(db, todo_id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {todo_id} not found"),
            });
        }
        let reminders = ReminderRepository::list_by_todo(db, todo_id)?;
        Ok(reminders.into_iter().map(ReminderDto::from).collect())
    }

    pub fn advance(db: &Database, reminder_id: &str) -> Result<(), AppError> {
        let reminder = ReminderRepository::get_by_id(db, reminder_id)?;
        ReminderRepository::advance(db, &reminder)?;
        let payload = serde_json::json!({
            "repeatType": reminder.repeat_type.as_str(),
        })
        .to_string();
        let _ = EventRepository::write(db, "reminder", reminder_id, "reminder.triggered", &payload);
        Ok(())
    }

    pub fn snooze(db: &Database, id: &str, minutes: i64) -> Result<ReminderDto, AppError> {
        if !SNOOZE_OPTIONS.contains(&minutes) {
            return Err(AppError::ValidationError {
                message: "snooze minutes must be one of 5, 15, 30, 60".into(),
            });
        }

        let reminder = ReminderRepository::snooze(db, id, minutes)?;
        let payload = serde_json::json!({ "minutes": minutes }).to_string();
        let _ = EventRepository::write(db, "reminder", id, "reminder.snoozed", &payload);
        Ok(reminder.into())
    }
}

fn parse_future_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AppError::ValidationError {
            message: format!("invalid datetime: {error}"),
        })
}

#[allow(dead_code)]
fn snooze_until(minutes: i64) -> DateTime<Utc> {
    Utc::now() + ChronoDuration::minutes(minutes)
}
