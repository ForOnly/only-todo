use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json;
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
    pub fn create(
        db: &Database,
        dto: &CreateTodoDto,
        priority: Priority,
    ) -> Result<Todo, AppError> {
        let now = Utc::now();
        let due_date = dto
            .due_date
            .as_ref()
            .map(|value| parse_optional_datetime(value))
            .transpose()?
            .flatten();
        let tags = dto.tags.clone().unwrap_or_default();
        let tags_json = serde_json::to_string(&tags).map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;

        let todo = Todo {
            id: Uuid::new_v4().to_string(),
            title: dto.title.clone(),
            description: dto.description.clone().unwrap_or_default(),
            status: TodoStatus::Todo,
            priority,
            due_date,
            tags,
            created_at: now,
            updated_at: now,
            completed_at: None,
            deleted_at: None,
        };

        db.with_tx(|conn| {
            conn.execute(
                "INSERT INTO todos (id, title, description, status, priority, due_date, tags, created_at, updated_at, completed_at, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    todo.id,
                    todo.title,
                    todo.description,
                    todo.status.as_str(),
                    todo.priority.as_str(),
                    todo.due_date.map(|value| value.to_rfc3339()),
                    tags_json,
                    todo.created_at.to_rfc3339(),
                    todo.updated_at.to_rfc3339(),
                    Option::<String>::None,
                    Option::<String>::None,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            sync_todo_tags(conn, &todo.id, &todo.tags)?;
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
        if let Some(due_date) = &dto.due_date {
            todo.due_date = match due_date {
                Some(value) => Some(parse_datetime(value)?),
                None => None,
            };
        }
        if let Some(tags) = &dto.tags {
            todo.tags = tags.clone();
        }
        todo.updated_at = Utc::now();

        let tags_json =
            serde_json::to_string(&todo.tags).map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;

        db.with_tx(|conn| {
            conn.execute(
                "UPDATE todos SET title = ?1, description = ?2, priority = ?3, due_date = ?4, tags = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    todo.title,
                    todo.description,
                    todo.priority.as_str(),
                    todo.due_date.map(|value| value.to_rfc3339()),
                    tags_json,
                    todo.updated_at.to_rfc3339(),
                    todo.id,
                ],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            if dto.tags.is_some() {
                sync_todo_tags(conn, &todo.id, &todo.tags)?;
            }
            Ok(todo)
        })
    }

    pub(crate) fn soft_delete_on(conn: &Connection, id: &str) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE todos SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
                params![now, id],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

        if rows == 0 {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }
        Ok(())
    }

    pub(crate) fn restore_on(conn: &Connection, id: &str) -> Result<Todo, AppError> {
        let now = Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE todos SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NOT NULL",
                params![now, id],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;

        if rows == 0 {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found or not deleted"),
            });
        }
        Self::get_by_id_on(conn, id)
    }

    pub fn get_by_id(db: &Database, id: &str) -> Result<Todo, AppError> {
        db.with_conn(|conn| Self::get_by_id_on(conn, id))
    }

    pub(crate) fn get_by_id_on(conn: &Connection, id: &str) -> Result<Todo, AppError> {
        conn.query_row(
            "SELECT id, title, description, status, priority, due_date, tags, created_at, updated_at, completed_at, deleted_at
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
    }

    pub fn transition(db: &Database, id: &str, status: TodoStatus) -> Result<Todo, AppError> {
        let mut todo = Self::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }

        let previous = todo.status;
        todo.status = status;
        todo.updated_at = Utc::now();
        todo.completed_at = if status == TodoStatus::Done {
            Some(Utc::now())
        } else if previous == TodoStatus::Done {
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

    pub fn count_active(db: &Database) -> Result<u64, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM todos WHERE deleted_at IS NULL AND status IN ('Todo', 'Doing')",
                [],
                |row| row.get(0),
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    /// 已逾期（due_date < 今日零点）的 Todo/Doing 任务数
    pub fn count_overdue(db: &Database, now_iso: &str) -> Result<u64, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM todos
                 WHERE deleted_at IS NULL AND status IN ('Todo', 'Doing')
                 AND due_date IS NOT NULL AND due_date < ?1",
                rusqlite::params![now_iso],
                |row| row.get(0),
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    /// 今日到期（due_date 在今日范围内）的 Todo/Doing 任务数
    pub fn count_due_today(db: &Database, start_iso: &str, end_iso: &str) -> Result<u64, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM todos
                 WHERE deleted_at IS NULL AND status IN ('Todo', 'Doing')
                 AND due_date IS NOT NULL AND due_date >= ?1 AND due_date < ?2",
                rusqlite::params![start_iso, end_iso],
                |row| row.get(0),
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    pub fn list_all_tags(db: &Database) -> Result<Vec<String>, AppError> {
        db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT t.name FROM tags t
                     INNER JOIN todo_tags tt ON tt.tag_name = t.name
                     INNER JOIN todos todo ON todo.id = tt.todo_id AND todo.deleted_at IS NULL
                     ORDER BY t.name COLLATE NOCASE",
                )
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?;
            let mut tags = Vec::new();
            for row in rows {
                tags.push(row.map_err(|error| AppError::DbError {
                    message: error.to_string(),
                })?);
            }
            Ok(tags)
        })
    }

    /// 将 todos.tags JSON 回填到规范化表（幂等）。
    pub fn backfill_normalized_tags(db: &Database) -> Result<(), AppError> {
        db.with_tx(|conn| {
            let snapshots = {
                let mut stmt = conn
                    .prepare("SELECT id, tags FROM todos")
                    .map_err(|error| AppError::DbError {
                        message: error.to_string(),
                    })?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(|error| AppError::DbError {
                        message: error.to_string(),
                    })?;
                let mut snapshots = Vec::new();
                for row in rows {
                    snapshots.push(row.map_err(|error| AppError::DbError {
                        message: error.to_string(),
                    })?);
                }
                snapshots
            };
            for (id, json) in snapshots {
                let tags: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
                sync_todo_tags(conn, &id, &tags)?;
            }
            Ok(())
        })
    }

    pub fn list(db: &Database, query: &ListTodoQuery) -> Result<PaginatedResponse<Todo>, AppError> {
        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();

        if query.include_deleted.unwrap_or(false) {
            conditions.push("deleted_at IS NOT NULL".to_string());
        } else {
            conditions.push("deleted_at IS NULL".to_string());
        }

        if let Some(statuses) = &query.status {
            if !statuses.is_empty() {
                let placeholders: Vec<String> = statuses.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("status IN ({})", placeholders.join(", ")));
                for status in statuses {
                    bind_values.push(status.as_str().to_string());
                }
            }
        } else if !query.include_archived.unwrap_or(false) && !query.include_deleted.unwrap_or(false)
        {
            conditions.push("status != 'Archived'".to_string());
        }

        if let Some(priorities) = &query.priority {
            if !priorities.is_empty() {
                let placeholders: Vec<String> =
                    priorities.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("priority IN ({})", placeholders.join(", ")));
                for priority in priorities {
                    bind_values.push(priority.as_str().to_string());
                }
            }
        }

        if let Some(tags) = &query.tags {
            for tag in tags {
                conditions.push(
                    "EXISTS (SELECT 1 FROM todo_tags tt WHERE tt.todo_id = todos.id AND tt.tag_name = ? COLLATE NOCASE)"
                        .to_string(),
                );
                bind_values.push(tag.clone());
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

        if let Some(after) = &query.due_date_after {
            conditions.push("due_date IS NOT NULL AND due_date >= ?".to_string());
            bind_values.push(after.clone());
        }

        if let Some(before) = &query.due_date_before {
            // 半开区间上界，与 count_overdue / count_due_today 一致
            conditions.push("due_date IS NOT NULL AND due_date < ?".to_string());
            bind_values.push(before.clone());
        }

        let where_clause = conditions.join(" AND ");
        let order_clause = build_order_clause(&query.sort_by, &query.sort_order);
        let offset = (query.page.saturating_sub(1) as u64) * query.page_size as u64;

        let count_sql = format!("SELECT COUNT(*) FROM todos WHERE {where_clause}");
        let list_sql = format!(
            "SELECT id, title, description, status, priority, due_date, tags, created_at, updated_at, completed_at, deleted_at
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

            let mut list_stmt = conn.prepare(&list_sql).map_err(|error| AppError::DbError {
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

fn sync_todo_tags(
    conn: &rusqlite::Connection,
    todo_id: &str,
    tags: &[String],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM todo_tags WHERE todo_id = ?1",
        params![todo_id],
    )
    .map_err(|error| AppError::DbError {
        message: error.to_string(),
    })?;

    let now = Utc::now().to_rfc3339();
    for tag in tags {
        let name = tag.trim();
        if name.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        conn.execute(
            "INSERT OR IGNORE INTO todo_tags (todo_id, tag_name) VALUES (?1, ?2)",
            params![todo_id, name],
        )
        .map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
    }
    Ok(())
}

fn build_order_clause(sort_by: &str, sort_order: &str) -> String {
    let direction = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    match sort_by {
        "created_at" | "createdAt" => format!("created_at {direction}"),
        "updated_at" | "updatedAt" => format!("updated_at {direction}"),
        "completed_at" | "completedAt" => format!("completed_at {direction}"),
        "due_date" | "dueDate" => format!("due_date IS NULL, due_date {direction}"),
        "title" => format!("title {direction}"),
        _ => format!(
            "CASE priority WHEN 'Urgent' THEN 4 WHEN 'High' THEN 3 WHEN 'Medium' THEN 2 WHEN 'Low' THEN 1 END {direction}, created_at DESC"
        ),
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    let status_str: String = row.get(3)?;
    let priority_str: String = row.get(4)?;
    let due_date: Option<String> = row.get(5)?;
    let tags_json: String = row.get(6)?;
    let created_at: String = row.get(7)?;
    let updated_at: String = row.get(8)?;
    let completed_at: Option<String> = row.get(9)?;
    let deleted_at: Option<String> = row.get(10)?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: TodoStatus::from_str(&status_str).unwrap_or(TodoStatus::Todo),
        priority: Priority::from_str(&priority_str).unwrap_or(Priority::Medium),
        due_date: due_date.map(|value| parse_datetime_unchecked(&value)),
        tags,
        created_at: parse_datetime_unchecked(&created_at),
        updated_at: parse_datetime_unchecked(&updated_at),
        completed_at: completed_at.map(|value| parse_datetime_unchecked(&value)),
        deleted_at: deleted_at.map(|value| parse_datetime_unchecked(&value)),
    })
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AppError::ValidationError {
            message: format!("invalid datetime: {error}"),
        })
}

fn parse_optional_datetime(value: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(parse_datetime(value)?))
}

fn parse_datetime_unchecked(value: &str) -> DateTime<Utc> {
    parse_datetime(value).unwrap_or_else(|_| Utc::now())
}
