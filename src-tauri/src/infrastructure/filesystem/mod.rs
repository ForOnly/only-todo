use std::path::PathBuf;

use tauri::Manager;

use crate::errors::AppError;

pub fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("only-todo"))
        .map_err(|error| AppError::InternalError {
            message: format!("failed to resolve app data dir: {error}"),
        })
}

pub fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    Ok(app_data_dir(app)?.join("data.db"))
}

pub fn logs_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    Ok(app_data_dir(app)?.join("logs"))
}

pub fn ensure_dirs(app: &tauri::AppHandle) -> Result<(), AppError> {
    let data_dir = app_data_dir(app)?;
    let logs = logs_dir(app)?;
    std::fs::create_dir_all(&data_dir).map_err(|error| AppError::InternalError {
        message: format!("failed to create data dir: {error}"),
    })?;
    std::fs::create_dir_all(&logs).map_err(|error| AppError::InternalError {
        message: format!("failed to create logs dir: {error}"),
    })?;
    Ok(())
}
