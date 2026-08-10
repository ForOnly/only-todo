use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::domain::{
    priority::Priority,
    status::TodoStatus,
    todo::{CreateTodoDto, ListTodoQuery, PaginatedResponse, Todo, UpdateTodoDto},
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;

pub struct TodoRepository;

impl TodoRepository {
    pub fn create(db: &Database, dto: &CreateTodoDto, priority: Priority) -> Result<Todo, AppError> {
        let now = Utc::now();
        let todo = Todo {
            id: Uuid::new_v4().to_string(),
            title: dto.title.clone(),
            description: dto.description.clone().unwrap_or_default(),
            status: TodoStatus::Todo,
            priority,
            created_at: now,
            updated_at: now,
            completed_at: None,
            deleted_at: None,
        };

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO todos (id, title, description, status, priority, created_at, updated_at, completed_at, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    todo.id,
                    todo.title,
                    todo.description,
                    todo.status.as_str(),
                    todo.priority.as_str(),
                    todo.created_at.to_rfc3339(),
                    todo.updated_at.to_rfc3339(),
                    Option::<String>::None,
                    Option::<String>::None,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(todo)
        })
    }

    pub fn update(db: &Database, dto: &UpdateTodoDto) -> Result<Todo, AppError> {
        let mut todo = Self::get_by_id(db, &dto.id)?;

        if let Some(title) = &dto.title {
            todo.title = title.clone();
        }
        if let Some(description) = &dto.description {
            todo.description = description.clone();
        }
        if let Some(priority) = dto.priority {
            todo.priority = priority;
        }
        todo.updated_at = Utc::now();

        db.with_conn(|conn| {
            conn.execute(
                "UPDATE todos SET title = ?1, description = ?2, priority = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    todo.title,
                    todo.description,
                    todo.priority.as_str(),
                    todo.updated_at.to_rfc3339(),
                    todo.id,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(todo)
        })
    }

    pub fn soft_delete(db: &Database, id: &str) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let rows = db.with_conn(|conn| {
            conn.execute(
                "UPDATE todos SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
                params![now, id],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })?;

        if rows == 0 {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }
        Ok(())
    }

    pub fn restore(db: &Database, id: &str) -> Result<Todo, AppError> {
        let now = Utc::now().to_rfc3339();
        let rows = db.with_conn(|conn| {
            conn.execute(
                "UPDATE todos SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NOT NULL",
                params![now, id],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })?;

        if rows == 0 {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found or not deleted"),
            });
        }
        Self::get_by_id(db, id)
    }

    pub fn get_by_id(db: &Database, id: &str) -> Result<Todo, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, title, description, status, priority, created_at, updated_at, completed_at, deleted_at
                 FROM todos WHERE id = ?1",
                params![id],
                map_row,
            )
            .optional()
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?
            .ok_or_else(|| AppError::NotFound {
                message: format!("todo {id} not found"),
            })
        })
    }

    pub fn transition(db: &Database, id: &str, status: TodoStatus) -> Result<Todo, AppError> {
        let mut todo = Self::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }

        todo.status = status;
        todo.updated_at = Utc::now();
        todo.completed_at = if status == TodoStatus::Done {
            Some(Utc::now())
        } else if status == TodoStatus::Todo {
            None
        } else {
            todo.completed_at
        };

        db.with_conn(|conn| {
            conn.execute(
                "UPDATE todos SET status = ?1, updated_at = ?2, completed_at = ?3 WHERE id = ?4",
                params![
                    todo.status.as_str(),
                    todo.updated_at.to_rfc3339(),
                    todo.completed_at.map(|value| value.to_rfc3339()),
                    todo.id,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(todo)
        })
    }

    pub fn list(db: &Database, query: &ListTodoQuery) -> Result<PaginatedResponse<Todo>, AppError> {
        let mut conditions = vec!["deleted_at IS NULL".to_string()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(statuses) = &query.status {
            if !statuses.is_empty() {
                let placeholders: Vec<String> =
                    statuses.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!(
                    "status IN ({})",
                    placeholders.join(", ")
                ));
                for status in statuses {
                    bind_values.push(status.as_str().to_string());
                }
            }
        }

        if let Some(priorities) = &query.priority {
            if !priorities.is_empty() {
                let placeholders: Vec<String> =
                    priorities.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!(
                    "priority IN ({})",
                    placeholders.join(", ")
                ));
                for priority in priorities {
                    bind_values.push(priority.as_str().to_string());
                }
            }
        }

        if let Some(keyword) = &query.keyword {
            if !keyword.trim().is_empty() {
                conditions.push("(title LIKE ? OR description LIKE ?)".to_string());
                let pattern = format!("%{}%", keyword.trim());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }

        let where_clause = conditions.join(" AND ");
        let order_clause = build_order_clause(&query.sort_by, &query.sort_order);
        let offset = (query.page.saturating_sub(1) as u64) * query.page_size as u64;

        let count_sql = format!("SELECT COUNT(*) FROM todos WHERE {where_clause}");
        let list_sql = format!(
            "SELECT id, title, description, status, priority, created_at, updated_at, completed_at, deleted_at
             FROM todos WHERE {where_clause} ORDER BY {order_clause} LIMIT ? OFFSET ?"
        );

        db.with_conn(|conn| {
            let mut count_stmt = conn
                .prepare(&count_sql)
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;
            let total: u64 = count_stmt
                .query_row(rusqlite::params_from_iter(bind_values.iter()), |row| {
                    row.get(0)
                })
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;

            let mut list_stmt = conn
                .prepare(&list_sql)
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;

            let mut list_bind: Vec<String> = bind_values;
            list_bind.push(query.page_size.to_string());
            list_bind.push(offset.to_string());

            let items = list_stmt
                .query_map(rusqlite::params_from_iter(list_bind.iter()), map_row)
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;

            Ok(PaginatedResponse {
                items,
                total,
                page: query.page,
                page_size: query.page_size,
            })
        })
    }
}

fn build_order_clause(sort_by: &str, sort_order: &str) -> String {
    let direction = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    match sort_by {
        "created_at" => format!("created_at {direction}"),
        "updated_at" => format!("updated_at {direction}"),
        "completed_at" => format!("completed_at {direction}"),
        "title" => format!("title {direction}"),
        _ => format!(
            "CASE priority WHEN 'Urgent' THEN 4 WHEN 'High' THEN 3 WHEN 'Medium' THEN 2 WHEN 'Low' THEN 1 END {direction}, created_at DESC"
        ),
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    let status_str: String = row.get(3)?;
    let priority_str: String = row.get(4)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;
    let completed_at: Option<String> = row.get(7)?;
    let deleted_at: Option<String> = row.get(8)?;

    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: TodoStatus::from_str(&status_str).unwrap_or(TodoStatus::Todo),
        priority: Priority::from_str(&priority_str).unwrap_or(Priority::Medium),
        created_at: parse_datetime(&created_at),
        updated_at: parse_datetime(&updated_at),
        completed_at: completed_at.map(|value| parse_datetime(&value)),
        deleted_at: deleted_at.map(|value| parse_datetime(&value)),
    })
}

fn parse_datetime(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
