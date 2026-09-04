use tauri::{AppHandle, Emitter, Manager, State};

use crate::domain::{BodyView, CompanionDragEndResult, CompanionSession, CompanionSurface};
use crate::errors::AppError;
use crate::float::host::FloatHost;
use crate::services::todo_service::TodoService;
use crate::state::AppState;

#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle, todo_id: Option<String>) -> Result<(), AppError> {
    show_main_window_impl(app, todo_id)
}

#[tauri::command]
pub fn show_floating_window(app: AppHandle) -> Result<(), AppError> {
    show_floating_window_impl(&app)?;
    Ok(())
}

#[tauri::command]
pub fn hide_floating_window(app: AppHandle) -> Result<(), AppError> {
    hide_floating_window_impl(&app)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_floating_window(app: AppHandle) -> Result<(), AppError> {
    toggle_floating_window_impl(&app)?;
    Ok(())
}

#[tauri::command]
pub fn get_active_todo_count(state: State<AppState>) -> Result<u64, AppError> {
    TodoService::count_active(&state.db)
}

#[tauri::command]
pub fn get_companion_session(app: AppHandle) -> Result<CompanionSession, AppError> {
    crate::float::snapshot(&app)
}

#[tauri::command]
pub fn companion_refresh_session(app: AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::publish(&app)
}

#[tauri::command]
pub fn companion_click_chrome(app: AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::click_chrome(&app)
}

#[tauri::command]
pub fn companion_minimize(app: AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::minimize(&app)
}

#[tauri::command]
pub fn companion_collapse_to_strip(app: AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::collapse_to_strip(&app)
}

#[tauri::command]
pub fn companion_drag_ended(
    app: AppHandle,
    which: CompanionSurface,
) -> Result<CompanionDragEndResult, AppError> {
    FloatHost::drag_ended(&app, which)
}

/// 悬停预览指针簇。`hold` 为快加草稿保活；inside/focused 仍如实写入。
#[tauri::command]
pub fn companion_pointer_cluster(
    app: AppHandle,
    surface: CompanionSurface,
    inside: Option<bool>,
    focused: Option<bool>,
    hold: Option<bool>,
) -> Result<CompanionSession, AppError> {
    FloatHost::pointer_cluster(&app, surface, inside, focused, hold)
}

#[tauri::command]
pub fn companion_open_view(app: AppHandle, view: BodyView) -> Result<CompanionSession, AppError> {
    FloatHost::open_view(&app, view)
}

/// 通知点击与托盘共用的主窗口显示逻辑（主窗优先：先退化助理钉住面板）。
pub fn show_main_window_impl(app: AppHandle, todo_id: Option<String>) -> Result<(), AppError> {
    // F1：先完成面板退化（0 或 1 次 apply），再 show/focus 主窗一次。
    // 退化失败不阻断出主窗（打 warn），避免单实例/通知路径整段失败。
    if let Err(error) = FloatHost::collapse_panel_for_main(&app) {
        tracing::warn!(error = %error, "collapse_panel_for_main failed");
    }

    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
        window.unminimize().ok();
        window.set_focus().ok();
    }

    if let Some(id) = todo_id {
        app.emit("navigate-to-todo", id)
            .map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;
    }

    Ok(())
}

/// 托盘左键：显隐主窗口。显示时会退化助理钉住面板（球/条可留）。
/// 可见（含系统最小化）→ `hide()` 收入托盘；已隐藏 → show/unminimize/focus。
pub fn toggle_main_window_impl(app: &AppHandle) -> Result<(), AppError> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    // 最小化时 is_visible 仍可能为 true：一律视为在场，收入托盘而非还原
    let visible = window.is_visible().unwrap_or(false);
    if visible {
        window.hide().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    } else {
        show_main_window_impl(app.clone(), None)?;
    }
    Ok(())
}

pub fn show_floating_window_impl(app: &AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::show(app)
}

pub fn hide_floating_window_impl(app: &AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::hide(app)
}

pub fn toggle_floating_window_impl(app: &AppHandle) -> Result<CompanionSession, AppError> {
    FloatHost::toggle(app)
}

#[tauri::command]
pub fn health_check() -> Result<String, AppError> {
    Ok("ok".into())
}
