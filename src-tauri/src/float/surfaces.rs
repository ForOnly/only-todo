use tauri::{LogicalPosition, LogicalSize, Manager, WebviewWindow};

use crate::domain::WindowBounds;
use crate::errors::AppError;

use super::{BODY_LABEL, CHROME_LABEL};

pub fn chrome_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(CHROME_LABEL)
}

pub fn body_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(BODY_LABEL)
}

pub fn apply_position(window: &WebviewWindow, x: f64, y: f64) -> Result<(), AppError> {
    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    Ok(())
}

pub fn apply_bounds(
    window: &WebviewWindow,
    bounds: &WindowBounds,
    visible: bool,
) -> Result<(), AppError> {
    let _ = window.set_shadow(false);
    window
        .set_size(LogicalSize::new(bounds.width, bounds.height))
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    apply_position(window, bounds.x, bounds.y)?;
    if visible {
        window.show().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    } else {
        window.hide().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    }
    Ok(())
}

pub fn hide_window(window: &WebviewWindow) -> Result<(), AppError> {
    window.hide().map_err(|error| AppError::InternalError {
        message: error.to_string(),
    })
}

pub fn set_always_on_top(app: &tauri::AppHandle, value: bool) {
    if let Some(window) = chrome_window(app) {
        let _ = window.set_always_on_top(value);
    }
    if let Some(window) = body_window(app) {
        let _ = window.set_always_on_top(value);
    }
}
