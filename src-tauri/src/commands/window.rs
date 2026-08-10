use tauri::{AppHandle, Emitter, Manager};

use crate::errors::AppError;

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

/// Tauri Command 与通知点击共用的主窗口显示逻辑。
pub fn show_main_window_impl(app: AppHandle, todo_id: Option<String>) -> Result<(), AppError> {
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

#[tauri::command]
pub fn health_check() -> Result<String, AppError> {
    Ok("ok".into())
}
