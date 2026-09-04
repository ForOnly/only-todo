use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod priority;
pub mod reminder;
pub mod status;
pub mod todo;

use todo::WorkbenchView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
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

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ball" => Some(Self::Ball),
            "panel" => Some(Self::Panel),
            "docked" => Some(Self::Ball),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
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

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "top" | "bottom" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum CompanionVisibility {
    Shown,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
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

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "free" | "ball" | "panel" => Some(Self::Free),
            "docked" => Some(Self::Docked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PanelMode {
    Closed,
    Preview,
    Pinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ChromeKind {
    Ball,
    Strip,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum BodyView {
    TodoMini,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum CompanionSurface {
    Chrome,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UiTheme {
    System,
    Light,
    Dark,
}

impl UiTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    /// 偏好 × OS → 实际浅/深。system 跟随 os，light/dark 钉死。
    pub fn resolve(self, os: ColorScheme) -> ColorScheme {
        match self {
            Self::Light => ColorScheme::Light,
            Self::Dark => ColorScheme::Dark,
            Self::System => os,
        }
    }
}

/// 已解析的浅/深（运行时，不落库）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppearanceDto {
    pub preference: UiTheme,
    pub resolved: ColorScheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum UiLocale {
    #[serde(rename = "zh-CN")]
    #[ts(rename = "zh-CN")]
    ZhCN,
    #[serde(rename = "en-US")]
    #[ts(rename = "en-US")]
    EnUS,
}

impl UiLocale {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhCN => "zh-CN",
            Self::EnUS => "en-US",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "en-US" | "en" => Self::EnUS,
            _ => Self::ZhCN,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct SettingsDto {
    pub notification_enabled: bool,
    pub list_default_sort: String,
    pub list_default_view: WorkbenchView,
    pub float_always_on_top: bool,
    #[ts(type = "number")]
    pub float_visible_count: u32,
    pub float_auto_show: bool,
    pub float_default_mode: HomeShape,
    pub float_hover_preview: bool,
    pub autostart_enabled: bool,
    pub ui_theme: UiTheme,
    pub ui_locale: UiLocale,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct UpdateSettingsDto {
    #[serde(default)]
    pub notification_enabled: Option<bool>,
    #[serde(default)]
    pub list_default_sort: Option<String>,
    #[serde(default)]
    pub list_default_view: Option<WorkbenchView>,
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
    #[serde(default)]
    pub ui_theme: Option<UiTheme>,
    #[serde(default)]
    pub ui_locale: Option<UiLocale>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct CompanionDragEndResult {
    pub still_dragging: bool,
    pub session: CompanionSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct CompanionSession {
    pub visibility: CompanionVisibility,
    pub placement: CompanionPlacement,
    pub home_shape: HomeShape,
    pub panel_mode: PanelMode,
    pub chrome: ChromeKind,
    pub dock_edge: DockEdge,
    #[ts(type = "number")]
    pub dock_y: f64,
    pub body_view: BodyView,
    pub hover_preview: bool,
    #[ts(type = "number")]
    pub active_count: u64,
    #[ts(type = "number")]
    pub overdue_count: u64,
    #[ts(type = "number")]
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

/// 导出前端共享类型（供 `cargo run --bin export-ts` 调用）
pub fn export_all_ts(out_dir: &std::path::Path) {
    use ts_rs::TS;

    std::fs::create_dir_all(out_dir).expect("create TS export dir");
    // SAFETY: 仅导出工具进程设置；不影响运行时应用
    std::env::set_var("TS_RS_EXPORT_DIR", out_dir);

    crate::domain::todo::TodoDto::export_all().expect("export TodoDto");
    crate::domain::todo::CreateTodoDto::export_all().expect("export CreateTodoDto");
    crate::domain::todo::UpdateTodoDto::export_all().expect("export UpdateTodoDto");
    crate::domain::todo::ListTodoQuery::export_all().expect("export ListTodoQuery");
    crate::domain::todo::ListWorkbenchQuery::export_all().expect("export ListWorkbenchQuery");
    crate::domain::todo::ListFocusBoardQuery::export_all().expect("export ListFocusBoardQuery");
    crate::domain::todo::FocusBoardDto::export_all().expect("export FocusBoardDto");
    crate::domain::todo::PaginatedTodos::export_all().expect("export PaginatedTodos");
    crate::domain::reminder::ReminderDto::export_all().expect("export ReminderDto");
    crate::domain::reminder::CreateReminderDto::export_all().expect("export CreateReminderDto");
    crate::domain::reminder::UpdateReminderDto::export_all().expect("export UpdateReminderDto");
    crate::domain::status::StatusActionDto::export_all().expect("export StatusActionDto");
    SettingsDto::export_all().expect("export SettingsDto");
    UpdateSettingsDto::export_all().expect("export UpdateSettingsDto");
    UiTheme::export_all().expect("export UiTheme");
    ColorScheme::export_all().expect("export ColorScheme");
    AppearanceDto::export_all().expect("export AppearanceDto");
    UiLocale::export_all().expect("export UiLocale");
    CompanionSession::export_all().expect("export CompanionSession");
    CompanionDragEndResult::export_all().expect("export CompanionDragEndResult");
    CompanionSurface::export_all().expect("export CompanionSurface");
    crate::events::EventDto::export_all().expect("export EventDto");
    crate::events::ListEventsQuery::export_all().expect("export ListEventsQuery");
    crate::events::WorkbenchMetaDto::export_all().expect("export WorkbenchMetaDto");

    write_generated_index(out_dir);
}

fn write_generated_index(out_dir: &std::path::Path) {
    let index = r#"/* eslint-disable */
/**
 * 由 `cargo run --bin export-ts` 从 Rust domain 生成。请勿手改具体类型文件；可改本 index 聚合。
 */
export type { TodoDto } from "./TodoDto";
export type { CreateTodoDto } from "./CreateTodoDto";
export type { UpdateTodoDto } from "./UpdateTodoDto";
export type { ListTodoQuery } from "./ListTodoQuery";
export type { ListWorkbenchQuery } from "./ListWorkbenchQuery";
export type { ListFocusBoardQuery } from "./ListFocusBoardQuery";
export type { FocusBoardDto } from "./FocusBoardDto";
export type { WorkbenchMetaDto } from "./WorkbenchMetaDto";
export type { WorkbenchView } from "./WorkbenchView";
export type { PaginatedTodos } from "./PaginatedTodos";
export type { TodoStatus } from "./TodoStatus";
export type { Priority } from "./Priority";
export type { ReminderDto } from "./ReminderDto";
export type { CreateReminderDto } from "./CreateReminderDto";
export type { UpdateReminderDto } from "./UpdateReminderDto";
export type { RepeatType } from "./RepeatType";
export type { StatusActionDto } from "./StatusActionDto";
export type { SettingsDto } from "./SettingsDto";
export type { UpdateSettingsDto } from "./UpdateSettingsDto";
export type { UiTheme } from "./UiTheme";
export type { ColorScheme } from "./ColorScheme";
export type { AppearanceDto } from "./AppearanceDto";
export type { UiLocale } from "./UiLocale";
export type { CompanionSession } from "./CompanionSession";
export type { CompanionDragEndResult } from "./CompanionDragEndResult";
export type { HomeShape } from "./HomeShape";
export type { DockEdge } from "./DockEdge";
export type { CompanionVisibility } from "./CompanionVisibility";
export type { CompanionPlacement } from "./CompanionPlacement";
export type { PanelMode } from "./PanelMode";
export type { ChromeKind } from "./ChromeKind";
export type { BodyView } from "./BodyView";
export type { CompanionSurface } from "./CompanionSurface";
export type { EventDto } from "./EventDto";
export type { ListEventsQuery } from "./ListEventsQuery";
"#;
    std::fs::write(out_dir.join("index.ts"), index).expect("write generated index");
}
