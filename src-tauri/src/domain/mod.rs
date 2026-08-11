use serde::{Deserialize, Serialize};

pub mod priority;
pub mod reminder;
pub mod status;
pub mod todo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FloatDisplayMode {
    Ball,
    Panel,
    Docked,
}

impl FloatDisplayMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ball => "ball",
            Self::Panel => "panel",
            Self::Docked => "docked",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ball" => Some(Self::Ball),
            "panel" => Some(Self::Panel),
            "docked" => Some(Self::Docked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl DockEdge {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub notification_enabled: bool,
    pub list_default_sort: String,
    pub float_always_on_top: bool,
    pub float_visible_count: u32,
    pub float_auto_show: bool,
    pub float_default_mode: FloatDisplayMode,
    pub autostart_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsDto {
    #[serde(default)]
    pub notification_enabled: Option<bool>,
    #[serde(default)]
    pub list_default_sort: Option<String>,
    #[serde(default)]
    pub float_always_on_top: Option<bool>,
    #[serde(default)]
    pub float_visible_count: Option<u32>,
    #[serde(default)]
    pub float_auto_show: Option<bool>,
    #[serde(default)]
    pub float_default_mode: Option<FloatDisplayMode>,
    #[serde(default)]
    pub autostart_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FloatWindowState {
    pub display_mode: FloatDisplayMode,
    pub default_mode: FloatDisplayMode,
    pub dock_edge: DockEdge,
    pub active_count: u64,
    pub overdue_count: u64,
    pub due_today_count: u64,
}
