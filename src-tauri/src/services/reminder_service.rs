use chrono::{DateTime, Utc};

use crate::domain::reminder::{
    CreateReminderDto, ReminderDto, RepeatType, UpdateReminderDto,
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::reminder_repository::ReminderRepository;
use crate::repository::todo_repository::TodoRepository;

const MAX_REMINDERS_PER_TODO: i64 = 5;

pub struct ReminderService;

impl ReminderService {
    pub fn create(db: &Database, dto: CreateReminderDto) -> Result<ReminderDto, AppError> {
        TodoRepository::get_by_id(db, &dto.todo_id)?;

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
        if repeat_type != RepeatType::None {
            return Err(AppError::ValidationError {
                message: "only repeat_type none is supported in v1.0".into(),
            });
        }

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
        TodoRepository::get_by_id(db, todo_id)?;
        let reminders = ReminderRepository::list_by_todo(db, todo_id)?;
        Ok(reminders.into_iter().map(ReminderDto::from).collect())
    }

    pub fn advance(db: &Database, reminder_id: &str) -> Result<(), AppError> {
        let reminder = ReminderRepository::get_by_id(db, reminder_id)?;
        ReminderRepository::advance(db, &reminder)
    }
}

fn parse_future_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AppError::ValidationError {
            message: format!("invalid datetime: {error}"),
        })
}
