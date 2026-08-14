use chrono::{Duration, Utc};

use crate::domain::{
    priority::Priority,
    status::{StatusActionDto, TodoStatus},
    todo::{
        CreateTodoDto, ListTodoQuery, ListWorkbenchQuery, PaginatedResponse, TodoDto,
        UpdateTodoDto, WorkbenchView,
    },
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
const TODAY_FETCH_LIMIT: u32 = 100;

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
        db.with_tx(|conn| {
            TodoRepository::soft_delete_on(conn, id)?;
            ReminderRepository::disable_by_todo_on(conn, id)?;
            Ok(())
        })
    }

    pub fn restore(db: &Database, id: &str) -> Result<TodoDto, AppError> {
        db.with_tx(|conn| {
            let todo = TodoRepository::restore_on(conn, id)?;
            ReminderRepository::reenable_after_restore_on(conn, id)?;
            Ok(todo.into())
        })
    }

    /// 含软删任务（回收站 Inspector）；update/transition 仍拒软删。
    pub fn get(db: &Database, id: &str) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::get_by_id(db, id)?;
        Ok(todo.into())
    }

    pub fn list(
        db: &Database,
        mut query: ListTodoQuery,
    ) -> Result<PaginatedResponse<TodoDto>, AppError> {
        normalize_page(&mut query.page, &mut query.page_size)?;
        let result = TodoRepository::list(db, &query)?;
        Ok(PaginatedResponse {
            items: result.items.into_iter().map(TodoDto::from).collect(),
            total: result.total,
            page: result.page,
            page_size: result.page_size,
        })
    }

    /// 工作台智能视图：视图语义集中在后端。
    pub fn list_workbench(
        db: &Database,
        query: ListWorkbenchQuery,
    ) -> Result<PaginatedResponse<TodoDto>, AppError> {
        let mut page = query.page;
        let mut page_size = query.page_size;
        normalize_page(&mut page, &mut page_size)?;

        let keyword = query.keyword.filter(|k| !k.trim().is_empty());

        match query.view {
            WorkbenchView::Today => {
                let now = Utc::now();
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::days(1);
                let due_query = ListTodoQuery {
                    status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
                    due_date_after: Some(start.to_rfc3339()),
                    due_date_before: Some(end.to_rfc3339()),
                    keyword: keyword.clone(),
                    sort_by: query.sort_by.clone(),
                    sort_order: query.sort_order.clone(),
                    page: 1,
                    page_size: TODAY_FETCH_LIMIT,
                    ..Default::default()
                };
                let doing_query = ListTodoQuery {
                    status: Some(vec![TodoStatus::Doing]),
                    keyword: keyword.clone(),
                    sort_by: query.sort_by.clone(),
                    sort_order: query.sort_order.clone(),
                    page: 1,
                    page_size: TODAY_FETCH_LIMIT,
                    ..Default::default()
                };
                let due = TodoRepository::list(db, &due_query)?;
                let doing = TodoRepository::list(db, &doing_query)?;
                let mut map = std::collections::HashMap::new();
                for todo in due.items.into_iter().chain(doing.items) {
                    map.insert(todo.id.clone(), todo);
                }
                let mut items: Vec<_> = map.into_values().collect();
                sort_todos_in_memory(&mut items, &query.sort_by, &query.sort_order);
                let total = items.len() as u64;
                Ok(PaginatedResponse {
                    items: items.into_iter().map(TodoDto::from).collect(),
                    total,
                    page: 1,
                    page_size: TODAY_FETCH_LIMIT,
                })
            }
            WorkbenchView::Overdue => {
                let start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                Self::list(
                    db,
                    ListTodoQuery {
                        status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
                        due_date_before: Some(start.to_rfc3339()),
                        keyword,
                        sort_by: query.sort_by,
                        sort_order: query.sort_order,
                        page,
                        page_size,
                        ..Default::default()
                    },
                )
            }
            WorkbenchView::Doing => Self::list(
                db,
                ListTodoQuery {
                    status: Some(vec![TodoStatus::Doing]),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
            WorkbenchView::All => Self::list(
                db,
                ListTodoQuery {
                    status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
            WorkbenchView::Done => Self::list(
                db,
                ListTodoQuery {
                    status: Some(vec![TodoStatus::Done]),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
            WorkbenchView::Archived => Self::list(
                db,
                ListTodoQuery {
                    status: Some(vec![TodoStatus::Archived]),
                    include_archived: Some(true),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
            WorkbenchView::Trash => Self::list(
                db,
                ListTodoQuery {
                    include_deleted: Some(true),
                    include_archived: Some(true),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
            WorkbenchView::Tag => Self::list(
                db,
                ListTodoQuery {
                    status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
                    tags: query.tag.map(|t| vec![t]),
                    keyword,
                    sort_by: query.sort_by,
                    sort_order: query.sort_order,
                    page,
                    page_size,
                    ..Default::default()
                },
            ),
        }
    }

    pub fn list_all_tags(db: &Database) -> Result<Vec<String>, AppError> {
        TodoRepository::list_all_tags(db)
    }

    pub fn count_active(db: &Database) -> Result<u64, AppError> {
        TodoRepository::count_active(db)
    }

    pub fn count_due_urgency(db: &Database) -> Result<(u64, u64), AppError> {
        let now = Utc::now();
        let start_of_today = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start = start_of_today.and_utc();
        let end = start + Duration::days(1);
        let overdue = TodoRepository::count_overdue(db, &start.to_rfc3339())?;
        let due_today =
            TodoRepository::count_due_today(db, &start.to_rfc3339(), &end.to_rfc3339())?;
        Ok((overdue, due_today))
    }

    pub fn allowed_transitions(status: TodoStatus) -> Vec<StatusActionDto> {
        status.allowed_transitions()
    }

    pub fn transition(db: &Database, id: &str, target: TodoStatus) -> Result<TodoDto, AppError> {
        let todo = TodoRepository::get_by_id(db, id)?;
        if todo.deleted_at.is_some() {
            return Err(AppError::NotFound {
                message: format!("todo {id} not found"),
            });
        }

        if !todo.status.can_transition_to(target) {
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

fn normalize_page(page: &mut u32, page_size: &mut u32) -> Result<(), AppError> {
    if *page == 0 {
        *page = 1;
    }
    if *page_size == 0 || *page_size > MAX_PAGE_SIZE {
        return Err(AppError::ValidationError {
            message: format!("page_size must be between 1 and {MAX_PAGE_SIZE}"),
        });
    }
    Ok(())
}

fn sort_todos_in_memory(
    items: &mut [crate::domain::todo::Todo],
    sort_by: &str,
    sort_order: &str,
) {
    let desc = sort_order.eq_ignore_ascii_case("desc");
    items.sort_by(|a, b| {
        let ord = match sort_by {
            "createdAt" | "created_at" => a.created_at.cmp(&b.created_at),
            "updatedAt" | "updated_at" => a.updated_at.cmp(&b.updated_at),
            "completedAt" | "completed_at" => a.completed_at.cmp(&b.completed_at),
            "dueDate" | "due_date" => a.due_date.cmp(&b.due_date),
            "title" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            _ => priority_rank(a.priority).cmp(&priority_rank(b.priority)),
        };
        if desc {
            ord.reverse()
        } else {
            ord
        }
    });
}

fn priority_rank(priority: Priority) -> u8 {
    match priority {
        Priority::Urgent => 4,
        Priority::High => 3,
        Priority::Medium => 2,
        Priority::Low => 1,
    }
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

impl Default for ListTodoQuery {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            tags: None,
            keyword: None,
            due_date_before: None,
            due_date_after: None,
            include_archived: None,
            include_deleted: None,
            sort_by: "priority".into(),
            sort_order: "desc".into(),
            page: 1,
            page_size: 50,
        }
    }
}
