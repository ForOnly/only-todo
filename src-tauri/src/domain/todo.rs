use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::priority::Priority;
use super::status::TodoStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct TodoDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
    pub priority: Priority,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl From<Todo> for TodoDto {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            status: todo.status,
            priority: todo.priority,
            due_date: todo.due_date.map(|value| value.to_rfc3339()),
            tags: todo.tags,
            created_at: todo.created_at.to_rfc3339(),
            updated_at: todo.updated_at.to_rfc3339(),
            completed_at: todo.completed_at.map(|value| value.to_rfc3339()),
            deleted_at: todo.deleted_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct CreateTodoDto {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct UpdateTodoDto {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    /// dueDate：缺省=不更新；null/空串=清空；字符串=设值（serde 侧处理）
    #[serde(default, deserialize_with = "deserialize_clearable_due_date")]
    #[ts(type = "string | null | undefined")]
    pub due_date: Option<Option<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WorkbenchView {
    Today,
    Overdue,
    Doing,
    All,
    Done,
    Archived,
    Trash,
    Tag,
}

impl WorkbenchView {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Overdue => "overdue",
            Self::Doing => "doing",
            Self::All => "all",
            Self::Done => "done",
            Self::Archived => "archived",
            Self::Trash => "trash",
            Self::Tag => "tag",
        }
    }

    /// settings `list.default_view`：非法或 tag → Today
    #[allow(clippy::should_implement_trait)]
    pub fn from_settings_str(value: &str) -> Self {
        match value {
            "overdue" => Self::Overdue,
            "doing" => Self::Doing,
            "all" => Self::All,
            "done" => Self::Done,
            "archived" => Self::Archived,
            "trash" => Self::Trash,
            "today" => Self::Today,
            _ => Self::Today,
        }
    }

    /// 可作为启动默认归类（排除 Tag）
    pub fn is_default_view_candidate(self) -> bool {
        !matches!(self, Self::Tag)
    }
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ListWorkbenchQuery {
    pub view: WorkbenchView,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
    #[serde(default = "default_page")]
    #[ts(type = "number")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    #[ts(type = "number")]
    pub page_size: u32,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ListTodoQuery {
    #[serde(default)]
    pub status: Option<Vec<TodoStatus>>,
    #[serde(default)]
    pub priority: Option<Vec<Priority>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub due_date_before: Option<String>,
    #[serde(default)]
    pub due_date_after: Option<String>,
    #[serde(default)]
    pub include_archived: Option<bool>,
    /// 为 true 时只列软删任务（回收站）
    #[serde(default)]
    pub include_deleted: Option<bool>,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
    #[serde(default = "default_page")]
    #[ts(type = "number")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    #[ts(type = "number")]
    pub page_size: u32,
}

fn default_sort_by() -> String {
    "priority".into()
}

fn default_sort_order() -> String {
    "desc".into()
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PaginatedTodos {
    pub items: Vec<TodoDto>,
    #[ts(type = "number")]
    pub total: u64,
    #[ts(type = "number")]
    pub page: u32,
    #[ts(type = "number")]
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl From<PaginatedResponse<TodoDto>> for PaginatedTodos {
    fn from(value: PaginatedResponse<TodoDto>) -> Self {
        Self {
            items: value.items,
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

/// dueDate：缺省=不更新；null/空串=清空；字符串=设值
fn deserialize_clearable_due_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct ClearableDueDateVisitor;

    impl<'de> Visitor<'de> for ClearableDueDateVisitor {
        type Value = Option<Option<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null, a string, or absent dueDate")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(None))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(None))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(StringOrNullVisitor)
        }
    }

    struct StringOrNullVisitor;

    impl<'de> Visitor<'de> for StringOrNullVisitor {
        type Value = Option<Option<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null or a string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(None))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(None))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(Some(None))
            } else {
                Ok(Some(Some(value.to_string())))
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(Some(None))
            } else {
                Ok(Some(Some(value)))
            }
        }
    }

    deserializer.deserialize_option(ClearableDueDateVisitor)
}
