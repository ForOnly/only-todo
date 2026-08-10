mod windows;

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::errors::AppError;
use crate::PendingNavigation;

pub use windows::ensure_toast_registration;

/// 提醒通知载荷：标题使用任务名以便在 toast 中突出显示。
#[derive(Debug, Clone)]
pub struct NotificationPayload {
    pub todo_id: String,
    pub title: String,
    pub trigger_at: DateTime<Utc>,
}

impl NotificationPayload {
    pub fn body(&self) -> String {
        let formatted = self.trigger_at.format("%m月%d日 %H:%M").to_string();
        format!("Only Todo · 提醒 · {formatted}")
    }
}

pub struct NotificationService;

impl NotificationService {
    /// 通过 tauri-plugin-notification 发送原生 toast（v1.0 Core 唯一展示通道）。
    pub fn send(app: &AppHandle, payload: NotificationPayload) -> Result<(), AppError> {
        let title = payload.title.clone();
        let body = payload.body();
        let todo_id = payload.todo_id.clone();

        app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|error| AppError::InternalError {
                message: format!("notification failed: {error}"),
            })?;

        set_pending_navigation(app, todo_id);
        tracing::info!("reminder notification sent: {}", payload.title);
        Ok(())
    }
}

/// 记录待跳转的任务 ID，窗口聚焦时由 lib.rs 发出 navigate-to-todo 事件。
fn set_pending_navigation(app: &AppHandle, todo_id: String) {
    if let Some(pending) = app.try_state::<PendingNavigation>() {
        if let Ok(mut guard) = pending.0.lock() {
            *guard = Some(todo_id);
        }
    }
}
