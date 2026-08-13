use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RepeatType {
    None,
    Daily,
    Weekly,
}

impl RepeatType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub todo_id: String,
    pub remind_at: DateTime<Utc>,
    pub repeat_type: RepeatType,
    pub repeat_config: String,
    pub snooze_count: i32,
    pub next_trigger_at: DateTime<Utc>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ReminderDto {
    pub id: String,
    pub todo_id: String,
    pub remind_at: String,
    pub repeat_type: RepeatType,
    pub repeat_config: String,
    #[ts(type = "number")]
    pub snooze_count: i32,
    pub next_trigger_at: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Reminder> for ReminderDto {
    fn from(reminder: Reminder) -> Self {
        Self {
            id: reminder.id,
            todo_id: reminder.todo_id,
            remind_at: reminder.remind_at.to_rfc3339(),
            repeat_type: reminder.repeat_type,
            repeat_config: reminder.repeat_config,
            snooze_count: reminder.snooze_count,
            next_trigger_at: reminder.next_trigger_at.to_rfc3339(),
            enabled: reminder.enabled,
            created_at: reminder.created_at.to_rfc3339(),
            updated_at: reminder.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct CreateReminderDto {
    pub todo_id: String,
    pub remind_at: String,
    #[serde(default)]
    pub repeat_type: Option<RepeatType>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct UpdateReminderDto {
    pub id: String,
    #[serde(default)]
    pub remind_at: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DueReminder {
    pub reminder: Reminder,
    pub todo_title: String,
    pub todo_deleted: bool,
}
