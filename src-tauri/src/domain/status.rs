use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "PascalCase")]
#[ts(export)]
pub enum TodoStatus {
    Todo,
    Doing,
    Done,
    Archived,
}

impl TodoStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "Todo",
            Self::Doing => "Doing",
            Self::Done => "Done",
            Self::Archived => "Archived",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "Todo" => Some(Self::Todo),
            "Doing" => Some(Self::Doing),
            "Done" => Some(Self::Done),
            "Archived" => Some(Self::Archived),
            _ => None,
        }
    }

    /// 状态机合法边（同态恒为 true）。
    pub fn can_transition_to(self, to: Self) -> bool {
        if self == to {
            return true;
        }
        matches!(
            (self, to),
            (Self::Todo, Self::Doing)
                | (Self::Todo, Self::Done)
                | (Self::Todo, Self::Archived)
                | (Self::Doing, Self::Todo)
                | (Self::Doing, Self::Done)
                | (Self::Doing, Self::Archived)
                | (Self::Done, Self::Todo)
                | (Self::Done, Self::Archived)
                | (Self::Archived, Self::Todo)
        )
    }

    /// Inspector 动词：合法目标状态（不含同态）。
    pub fn allowed_transitions(self) -> Vec<StatusActionDto> {
        match self {
            Self::Todo => vec![
                StatusActionDto::new("开始", Self::Doing),
                StatusActionDto::new("完成", Self::Done),
                StatusActionDto::new("归档", Self::Archived),
            ],
            Self::Doing => vec![
                StatusActionDto::new("回待办", Self::Todo),
                StatusActionDto::new("完成", Self::Done),
                StatusActionDto::new("归档", Self::Archived),
            ],
            Self::Done => vec![
                StatusActionDto::new("重开", Self::Todo),
                StatusActionDto::new("归档", Self::Archived),
            ],
            Self::Archived => vec![StatusActionDto::new("重开", Self::Todo)],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct StatusActionDto {
    pub label: String,
    pub target: TodoStatus,
}

impl StatusActionDto {
    fn new(label: &str, target: TodoStatus) -> Self {
        Self {
            label: label.to_string(),
            target,
        }
    }
}
