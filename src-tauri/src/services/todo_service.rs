use chrono::{DateTime, Duration, Utc};

use crate::domain::{
    priority::Priority,
    status::{StatusActionDto, TodoStatus},
    todo::{
        CreateTodoDto, FocusBoardDto, ListFocusBoardQuery, ListTimePlannerQuery, ListTodoQuery,
        ListWorkbenchQuery, PaginatedResponse, TimePlannerDto, TodoDto, TodoPlannerPreviewDto,
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

        let overdue = Self::list(
            db,
            ListTodoQuery {
                status: Some(vec![TodoStatus::Todo, TodoStatus::Doing]),
                due_date_before: Some(start.to_rfc3339()),
                sort_by: sort_by.clone(),
                sort_order: sort_order.clone(),
                page: 1,
                page_size: TODAY_FETCH_LIMIT,
                ..Default::default()
            },
        )?;

        let doing_all = Self::list(
            db,
            ListTodoQuery {
                status: Some(vec![TodoStatus::Doing]),
                sort_by: sort_by.clone(),
                sort_order: sort_order.clone(),
                page: 1,
                page_size: TODAY_FETCH_LIMIT,
                ..Default::default()
            },
        )?;

        let due_today = Self::list(
            db,
            ListTodoQuery {
                status: Some(vec![TodoStatus::Todo]),
                due_date_after: Some(start.to_rfc3339()),
                due_date_before: Some(end.to_rfc3339()),
                sort_by,
                sort_order,
                page: 1,
                page_size: TODAY_FETCH_LIMIT,
                ..Default::default()
            },
        )?;

        let overdue_ids: std::collections::HashSet<String> =
            overdue.items.iter().map(|todo| todo.id.clone()).collect();
        let doing = doing_all
            .items
            .into_iter()
            .filter(|todo| !overdue_ids.contains(&todo.id))
            .collect();

        Ok(FocusBoardDto {
            overdue: overdue.items,
            doing,
            due_today: due_today.items,
        })
    }

    /// 时间规划面板：接下来 / 逾期 / 最久未动
    pub fn list_time_planner(
        db: &Database,
        query: ListTimePlannerQuery,
    ) -> Result<TimePlannerDto, AppError> {
        let now = Utc::now();
        let start_of_today = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let range_end = now + Duration::days(query.range_days.max(1) as i64);
        let top_n = query.top_n.clamp(1, 20) as usize;

        let rows = TodoRepository::list_active_for_planner(db)?;
        let mut overdue = Vec::new();
        let mut next = Vec::new();
        let mut stale = Vec::new();

        for (todo, next_trigger) in rows {
            let preview = planner_preview_from(&todo, next_trigger);

            if let Some(due) = todo.due_date {
                if due < start_of_today {
                    overdue.push(preview.clone());
                }
            }

            if let Some(plan_at) = compute_plan_at(todo.due_date, next_trigger, now) {
                if plan_at <= range_end {
                    let mut item = preview.clone();
                    item.plan_at = Some(plan_at.to_rfc3339());
                    next.push(item);
                }
            }

            stale.push(preview);
        }

        overdue.sort_by(|a, b| a.due_date.cmp(&b.due_date));
        next.sort_by(|a, b| a.plan_at.cmp(&b.plan_at));
        stale.truncate(top_n);

        Ok(TimePlannerDto {
            next,
            overdue,
            stale,
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

fn planner_preview_from(
    todo: &crate::domain::todo::Todo,
    next_trigger: Option<DateTime<Utc>>,
) -> TodoPlannerPreviewDto {
    TodoPlannerPreviewDto {
        id: todo.id.clone(),
        title: todo.title.clone(),
        status: todo.status,
        priority: todo.priority,
        due_date: todo.due_date.map(|value| value.to_rfc3339()),
        created_at: todo.created_at.to_rfc3339(),
        updated_at: todo.updated_at.to_rfc3339(),
        tags: todo.tags.clone(),
        next_trigger_at: next_trigger.map(|value| value.to_rfc3339()),
        plan_at: None,
    }
}

fn compute_plan_at(
    due: Option<DateTime<Utc>>,
    next_trigger: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    let mut candidates = Vec::new();
    if let Some(due_at) = due {
        if due_at >= now {
            candidates.push(due_at);
        }
    }
    if let Some(trigger_at) = next_trigger {
        if trigger_at >= now {
            candidates.push(trigger_at);
        }
    }
    candidates.into_iter().min()
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
