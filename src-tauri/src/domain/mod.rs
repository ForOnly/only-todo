use serde::{Deserialize, Serialize};

pub mod priority;
pub mod reminder;
pub mod status;
pub mod todo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HomeShape {
    Ball,
    Panel,
}

impl HomeShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ball => "ball",
            Self::Panel => "panel",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ball" => Some(Self::Ball),
            "panel" => Some(Self::Panel),
            "docked" => Some(Self::Ball),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockEdge {
    Left,
    Right,
}

impl DockEdge {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "top" | "bottom" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompanionVisibility {
    Shown,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompanionPlacement {
    Free,
    Docked,
}

impl CompanionPlacement {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Docked => "docked",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "free" | "ball" | "panel" => Some(Self::Free),
            "docked" => Some(Self::Docked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelMode {
    Closed,
    Preview,
    Pinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChromeKind {
    Ball,
    Strip,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BodyView {
    TodoMini,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompanionSurface {
    Chrome,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub notification_enabled: bool,
    pub list_default_sort: String,
    pub float_always_on_top: bool,
    pub float_visible_count: u32,
    pub float_auto_show: bool,
    pub float_default_mode: HomeShape,
    pub float_hover_preview: bool,
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
    pub float_default_mode: Option<HomeShape>,
    #[serde(default)]
    pub float_hover_preview: Option<bool>,
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
pub struct CompanionDragEndResult {
    pub still_dragging: bool,
    pub session: CompanionSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSession {
    pub visibility: CompanionVisibility,
    pub placement: CompanionPlacement,
    pub home_shape: HomeShape,
    pub panel_mode: PanelMode,
    pub chrome: ChromeKind,
    pub dock_edge: DockEdge,
    pub dock_y: f64,
    pub body_view: BodyView,
    pub hover_preview: bool,
    pub active_count: u64,
    pub overdue_count: u64,
    pub due_today_count: u64,
}

pub fn derive_chrome(
    visibility: CompanionVisibility,
    placement: CompanionPlacement,
    home_shape: HomeShape,
    panel_mode: PanelMode,
) -> ChromeKind {
    if visibility != CompanionVisibility::Shown {
        return ChromeKind::Hidden;
    }
    match placement {
        CompanionPlacement::Docked => ChromeKind::Strip,
        CompanionPlacement::Free => {
            if panel_visible(placement, panel_mode) {
                ChromeKind::Hidden
            } else if home_shape == HomeShape::Ball {
                ChromeKind::Ball
            } else {
                ChromeKind::Hidden
            }
        }
    }
}

pub fn panel_visible(placement: CompanionPlacement, panel_mode: PanelMode) -> bool {
    match placement {
        CompanionPlacement::Free => panel_mode == PanelMode::Pinned,
        CompanionPlacement::Docked => panel_mode != PanelMode::Closed,
    }
}
