use crate::domain::{
    priority::Priority,
    status::TodoStatus,
    todo::{
        CreateTodoDto, ListTodoQuery, PaginatedResponse, Todo, TodoDto, UpdateTodoDto,
    },
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::todo_repository::TodoRepository;

const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 5000;
const MAX_PAGE_SIZE: u32 = 200;

pub struct TodoService;

impl TodoService {
    pub fn create(db: &Database, mut dto: CreateTodoDto) -> Result<TodoDto, AppError> {
        dto.title = dto.title.trim().to_string();
        validate_title(&dto.title)?;
        if let Some(description) = &dto.description {
            validate_description(description)?;
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

        let todo = TodoRepository::update(db, &dto)?;
        Ok(todo.into())
    }

    pub fn delete(db: &Database, id: &str) -> Result<(), AppError> {
        TodoRepository::soft_delete(db, id)
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

    pub fn transition(db: &Database, id: &str, target: TodoStatus) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }

        if !is_valid_v1_transition(todo.status, target) {
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

        let updated = TodoRepository::transition(db, id, target)?;
        Ok(updated.into())
    }
}

fn is_valid_v1_transition(from: TodoStatus, to: TodoStatus) -> bool {
    matches!(
        (from, to),
        (TodoStatus::Todo, TodoStatus::Done)
            | (TodoStatus::Done, TodoStatus::Todo)
            | (TodoStatus::Todo, TodoStatus::Todo)
            | (TodoStatus::Done, TodoStatus::Done)
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
