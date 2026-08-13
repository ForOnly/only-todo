//! 审计事件读取 API（v1.2+ 写入；本模块提供游标查询）

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::event_repository::EventRepository;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct EventDto {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub event_type: String,
    pub payload: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ListEventsQuery {
    /// 仅返回 created_at 严格大于该游标的事件（RFC3339）
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default = "default_limit")]
    #[ts(type = "number")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

pub fn list_events(db: &Database, query: ListEventsQuery) -> Result<Vec<EventDto>, AppError> {
    let limit = query.limit.clamp(1, 200);
    EventRepository::list_since(db, query.after.as_deref(), limit)
}
