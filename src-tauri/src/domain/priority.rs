use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "Low" => Some(Self::Low),
            "Medium" => Some(Self::Medium),
            "High" => Some(Self::High),
            "Urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}
