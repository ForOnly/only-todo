use chrono::{DateTime, Utc};
use serde_json;

use crate::domain::{
    priority::Priority,
    status::TodoStatus,
    todo::{CreateTodoDto, ListTodoQuery, PaginatedResponse, TodoDto, UpdateTodoDto},
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::event_repository::EventRepository;
use crate::repository::reminder_repository::ReminderRepository;
use crate::repository::todo_repository::TodoRepository;

const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 5000;
const MAX_PAGE_SIZE: u32 = 200;
const MAX_TAGS: usize = 10;
const MAX_TAG_LEN: usize = 30;

pub struct TodoService;

impl TodoService {
    pub fn create(db: &Database, mut dto: CreateTodoDto) -> Result<TodoDto, AppError> {
        dto.title = dto.title.trim().to_string();
        validate_title(&dto.title)?;
        if let Some(description) = &dto.description {
            validate_description(description)?;
        }
        if let Some(tags) = &dto.tags {
            validate_tags(tags)?;
        }

        let priority = dto.priority.unwrap_or(Priority::Medium);
        let todo = TodoRepository::create(db, &dto, priority)?;
        Ok(todo.into())
    }

    pub fn update(db: &Database, dto: UpdateTodoDto) -> Result<TodoDto, AppError> {
        if let Some(title) = &dto.title {
            validate_title(title)?;
        }
        if let Some(description) = &dto.description {
            validate_description(description)?;
        }
        if let Some(tags) = &dto.tags {
            validate_tags(tags)?;
        }

        let existing = TodoRepository::get_by_id(db, &dto.id)?;
        if existing.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {} not found", dto.id),
            });
        }

        let todo = TodoRepository::update(db, &dto)?;
        Ok(todo.into())
    }

    pub fn delete(db: &Database, id: &str) -> Result<(), AppError> {
        TodoRepository::soft_delete(db, id)?;
        ReminderRepository::disable_by_todo(db, id)?;
        Ok(())
    }

    pub fn restore(db: &Database, id: &str) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::restore(db, id)?;
        Ok(todo.into())
    }

    pub fn get(db: &Database, id: &str) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }
        Ok(todo.into())
    }

    pub fn list(
        db: &Database,
        mut query: ListTodoQuery,
    ) -> Result<PaginatedResponse<TodoDto>, AppError> {
        if query.page == 0 {
            query.page = 1;
        }
        if query.page_size == 0 || query.page_size > MAX_PAGE_SIZE {
            return Err(AppError::ValidationError {
                message: format!("page_size must be between 1 and {MAX_PAGE_SIZE}"),
            });
        }

        let result = TodoRepository::list(db, &query)?;
        Ok(PaginatedResponse {
            items: result.items.into_iter().map(TodoDto::from).collect(),
            total: result.total,
            page: result.page,
            page_size: result.page_size,
        })
    }

    pub fn list_all_tags(db: &Database) -> Result<Vec<String>, AppError> {
        TodoRepository::list_all_tags(db)
    }

    pub fn count_active(db: &Database) -> Result<u64, AppError> {
        TodoRepository::count_active(db)
    }

    pub fn count_due_urgency(db: &Database) -> Result<(u64, u64), AppError> {
        use chrono::{Duration, Utc};
        let now = Utc::now();
        let start_of_today = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start = start_of_today.and_utc();
        let end = start + Duration::days(1);
        let overdue = TodoRepository::count_overdue(db, &start.to_rfc3339())?;
        let due_today =
            TodoRepository::count_due_today(db, &start.to_rfc3339(), &end.to_rfc3339())?;
        Ok((overdue, due_today))
    }

    pub fn transition(db: &Database, id: &str, target: TodoStatus) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }

        if !is_valid_transition(todo.status, target) {
            return Err(AppError::InvalidTransition {
                message: format!(
                    "cannot transition from {} to {}",
                    todo.status.as_str(),
                    target.as_str()
                ),
            });
        }

        if todo.status == target {
            return Ok(todo.into());
        }

        let from = todo.status;
        let updated = TodoRepository::transition(db, id, target)?;
        let payload =
            serde_json::json!({ "from": from.as_str(), "to": target.as_str() }).to_string();
        let _ = EventRepository::write(db, "todo", id, "todo.transitioned", &payload);
        Ok(updated.into())
    }
}

fn is_valid_transition(from: TodoStatus, to: TodoStatus) -> bool {
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (TodoStatus::Todo, TodoStatus::Doing)
            | (TodoStatus::Todo, TodoStatus::Done)
            | (TodoStatus::Todo, TodoStatus::Archived)
            | (TodoStatus::Doing, TodoStatus::Todo)
            | (TodoStatus::Doing, TodoStatus::Done)
            | (TodoStatus::Doing, TodoStatus::Archived)
            | (TodoStatus::Done, TodoStatus::Todo)
            | (TodoStatus::Done, TodoStatus::Archived)
            | (TodoStatus::Archived, TodoStatus::Todo)
    )
}

fn validate_title(title: &str) -> Result<(), AppError> {
    let trimmed = title.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TITLE_LEN {
        return Err(AppError::ValidationError {
            message: format!("title must be 1-{MAX_TITLE_LEN} characters"),
        });
    }
    Ok(())
}

fn validate_description(description: &str) -> Result<(), AppError> {
    if description.len() > MAX_DESCRIPTION_LEN {
        return Err(AppError::ValidationError {
            message: format!("description must be at most {MAX_DESCRIPTION_LEN} characters"),
        });
    }
    Ok(())
}

fn validate_tags(tags: &[String]) -> Result<(), AppError> {
    if tags.len() > MAX_TAGS {
        return Err(AppError::ValidationError {
            message: format!("tags must have at most {MAX_TAGS} items"),
        });
    }
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_TAG_LEN {
            return Err(AppError::ValidationError {
                message: format!("each tag must be 1-{MAX_TAG_LEN} characters"),
            });
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn parse_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AppError::ValidationError {
            message: format!("invalid datetime: {error}"),
        })
}
