use chrono::{Duration, Utc};

use crate::domain::{
    priority::Priority,
    status::{StatusActionDto, TodoStatus},
    todo::{
        CreateTodoDto, FocusBoardDto, ListFocusBoardQuery, ListTodoQuery, ListWorkbenchQuery,
        PaginatedResponse, TodoDto, UpdateTodoDto, WorkbenchView,
    },
};
use crate::errors::AppError;
use crate::events::WorkbenchMetaDto;
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
                let board = Self::list_focus_board(
                    db,
                    ListFocusBoardQuery {
                        sort_by: query.sort_by,
                        sort_order: query.sort_order,
                    },
                )?;
                let items: Vec<TodoDto> = board
                    .overdue
                    .into_iter()
                    .chain(board.doing)
                    .chain(board.due_today)
                    .collect();
                let total = items.len() as u64;
                Ok(PaginatedResponse {
                    items,
                    total,
                    page: 1,
                    page_size: TODAY_FETCH_LIMIT,
                })
            }
            WorkbenchView::Overdue => {
                let start = Utc::now()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
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

    /// 焦点台：逾期 / 进行中（未逾期）/ 今日到期（仅 Todo），三组互斥。
    /// 单次持锁、三次 SELECT（无 COUNT）。
    pub fn list_focus_board(
        db: &Database,
        query: ListFocusBoardQuery,
    ) -> Result<FocusBoardDto, AppError> {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = start + Duration::days(1);
        let sort_by = if query.sort_by.trim().is_empty() {
            "priority".to_string()
        } else {
            query.sort_by
        };
        let sort_order = if query.sort_order.trim().is_empty() {
            "desc".to_string()
        } else {
            query.sort_order
        };

        let overdue_q = ListTodoQuery {
            status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
            due_date_before: Some(start.to_rfc3339()),
            sort_by: sort_by.clone(),
            sort_order: sort_order.clone(),
            page: 1,
            page_size: TODAY_FETCH_LIMIT,
            ..Default::default()
        };
        let doing_q = ListTodoQuery {
            status: Some(vec![TodoStatus::Doing]),
            sort_by: sort_by.clone(),
            sort_order: sort_order.clone(),
            page: 1,
            page_size: TODAY_FETCH_LIMIT,
            ..Default::default()
        };
        let due_today_q = ListTodoQuery {
            status: Some(vec![TodoStatus::Todo]),
            due_date_after: Some(start.to_rfc3339()),
            due_date_before: Some(end.to_rfc3339()),
            sort_by,
            sort_order,
            page: 1,
            page_size: TODAY_FETCH_LIMIT,
            ..Default::default()
        };

        db.with_conn(|conn| {
            let overdue_items = TodoRepository::list_items_on(conn, &overdue_q)?;
            let doing_all = TodoRepository::list_items_on(conn, &doing_q)?;
            let due_today_items = TodoRepository::list_items_on(conn, &due_today_q)?;

            let overdue_ids: std::collections::HashSet<String> =
                overdue_items.iter().map(|todo| todo.id.clone()).collect();
            let doing = doing_all
                .into_iter()
                .filter(|todo| !overdue_ids.contains(&todo.id))
                .map(TodoDto::from)
                .collect();

            Ok(FocusBoardDto {
                overdue: overdue_items.into_iter().map(TodoDto::from).collect(),
                doing,
                due_today: due_today_items.into_iter().map(TodoDto::from).collect(),
            })
        })
    }

    /// 主窗侧栏元数据：标签 + 最近活动，一次 IPC、单次持锁。
    pub fn workbench_meta(db: &Database, event_limit: u32) -> Result<WorkbenchMetaDto, AppError> {
        let limit = event_limit.clamp(1, 50);
        db.with_conn(|conn| {
            let tags = TodoRepository::list_all_tags_on(conn)?;
            // 与 list_events 无游标一致：最新 N 条再按时间正序；活动条前端再降序
            let events = EventRepository::list_since_on(conn, None, limit)?;
            Ok(WorkbenchMetaDto { tags, events })
        })
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
        let payload = serde_json::json!({
            "from": from.as_str(),
            "to": target.as_str(),
            "title": todo.title,
        })
        .to_string();
        if let Err(error) = EventRepository::write(db, "todo", id, "todo.transitioned", &payload) {
            tracing::warn!(
                error = %error,
                entity_id = id,
                "failed to write audit event"
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::todo::CreateTodoDto;

    fn db() -> Database {
        Database::open_in_memory().expect("open in-memory db")
    }

    fn create_titled(db: &Database, title: &str) -> TodoDto {
        TodoService::create(
            db,
            CreateTodoDto {
                title: title.into(),
                description: None,
                priority: None,
                due_date: None,
                tags: None,
            },
        )
        .expect("create todo")
    }

    #[test]
    fn transition_todo_to_doing_and_done() {
        let db = db();
        let todo = create_titled(&db, "t1");
        assert_eq!(todo.status, TodoStatus::Todo);

        let doing = TodoService::transition(&db, &todo.id, TodoStatus::Doing).unwrap();
        assert_eq!(doing.status, TodoStatus::Doing);

        let done = TodoService::transition(&db, &todo.id, TodoStatus::Done).unwrap();
        assert_eq!(done.status, TodoStatus::Done);
        assert!(done.completed_at.is_some());
    }

    #[test]
    fn transition_rejects_illegal_edge() {
        let db = db();
        let todo = create_titled(&db, "t2");
        TodoService::transition(&db, &todo.id, TodoStatus::Done).unwrap();
        let err = TodoService::transition(&db, &todo.id, TodoStatus::Doing).unwrap_err();
        match err {
            AppError::InvalidTransition { .. } => {}
            other => panic!("expected InvalidTransition, got {other:?}"),
        }
    }

    #[test]
    fn soft_delete_and_restore() {
        let db = db();
        let todo = create_titled(&db, "t3");
        TodoService::delete(&db, &todo.id).unwrap();
        let listed = TodoService::list(
            &db,
            ListTodoQuery {
                status: Some(vec![TodoStatus::Todo]),
                page: 1,
                page_size: 50,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(listed.items.iter().all(|t| t.id != todo.id));

        let restored = TodoService::restore(&db, &todo.id).unwrap();
        assert!(restored.deleted_at.is_none());
        assert_eq!(restored.title, "t3");
    }

    #[test]
    fn focus_board_partitions_overdue_doing_due_today() {
        let db = db();
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let yesterday = (start - Duration::days(1)).to_rfc3339();
        let today_noon = (start + Duration::hours(12)).to_rfc3339();

        let overdue = TodoService::create(
            &db,
            CreateTodoDto {
                title: "overdue".into(),
                description: None,
                priority: None,
                due_date: Some(yesterday),
                tags: None,
            },
        )
        .unwrap();
        TodoService::transition(&db, &overdue.id, TodoStatus::Doing).unwrap();

        let doing = TodoService::create(
            &db,
            CreateTodoDto {
                title: "doing".into(),
                description: None,
                priority: None,
                due_date: None,
                tags: None,
            },
        )
        .unwrap();
        TodoService::transition(&db, &doing.id, TodoStatus::Doing).unwrap();

        let due_today = TodoService::create(
            &db,
            CreateTodoDto {
                title: "due-today".into(),
                description: None,
                priority: None,
                due_date: Some(today_noon),
                tags: None,
            },
        )
        .unwrap();

        let board = TodoService::list_focus_board(
            &db,
            ListFocusBoardQuery {
                sort_by: "priority".into(),
                sort_order: "desc".into(),
            },
        )
        .unwrap();

        assert!(board.overdue.iter().any(|t| t.id == overdue.id));
        assert!(board.doing.iter().any(|t| t.id == doing.id));
        assert!(!board.doing.iter().any(|t| t.id == overdue.id));
        assert!(board.due_today.iter().any(|t| t.id == due_today.id));
    }

    #[test]
    fn list_omits_description_payload() {
        let db = db();
        let todo = TodoService::create(
            &db,
            CreateTodoDto {
                title: "with-desc".into(),
                description: Some("secret body".into()),
                priority: None,
                due_date: None,
                tags: None,
            },
        )
        .unwrap();
        assert_eq!(todo.description, "secret body");

        let listed = TodoService::list(
            &db,
            ListTodoQuery {
                page: 1,
                page_size: 50,
                ..Default::default()
            },
        )
        .unwrap();
        let row = listed.items.iter().find(|t| t.id == todo.id).unwrap();
        assert_eq!(row.description, "");

        let full = TodoService::get(&db, &todo.id).unwrap();
        assert_eq!(full.description, "secret body");
    }

    #[test]
    fn workbench_meta_returns_tags() {
        let db = db();
        TodoService::create(
            &db,
            CreateTodoDto {
                title: "tagged".into(),
                description: None,
                priority: None,
                due_date: None,
                tags: Some(vec!["alpha".into(), "beta".into()]),
            },
        )
        .unwrap();
        let meta = TodoService::workbench_meta(&db, 8).unwrap();
        assert!(meta.tags.iter().any(|t| t == "alpha"));
        assert!(meta.tags.iter().any(|t| t == "beta"));
        assert!(meta.events.is_empty() || meta.events.len() <= 8);
    }
}
