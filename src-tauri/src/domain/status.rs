use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
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

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "Todo" => Some(Self::Todo),
            "Doing" => Some(Self::Doing),
            "Done" => Some(Self::Done),
            "Archived" => Some(Self::Archived),
            _ => None,
        }
    }
}
